use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use tokio::sync::RwLock;
use rmcp::model::Tool;
use tracing::{info, warn};

use super::loader::JsonLoader;
use super::schema::AppError;
use super::skill_scanner::SkillInfo;
use super::mcp_tools;
use crate::db::{SkillDb, SkillStore};
use crate::exec::{ToolExecutor, ToolHandler};
use crate::exec::file_handler::FileHandlerFactory;

pub struct ToolManager {
    tools: Arc<RwLock<Vec<Tool>>>,
    executor: Arc<RwLock<ToolExecutor>>,
    loader: JsonLoader,
    skills_path: String,
    reload_flag: Arc<AtomicBool>,
    skills: Arc<RwLock<Vec<SkillInfo>>>,
    db: Arc<SkillDb>,
}

impl ToolManager {
    pub fn new(tools_path: &str, skills_path: &str, db_path: &str) -> Self {
        let reload_flag = Arc::new(AtomicBool::new(false));
        let db = SkillDb::new(db_path).expect("无法创建数据库");
        
        Self {
            tools: Arc::new(RwLock::new(Vec::new())),
            executor: Arc::new(RwLock::new(ToolExecutor::new())),
            loader: JsonLoader::new(tools_path),
            skills_path: skills_path.to_string(),
            reload_flag,
            skills: Arc::new(RwLock::new(Vec::new())),
            db: Arc::new(db),
        }
    }

    pub fn db(&self) -> &Arc<SkillDb> {
        &self.db
    }

    pub fn skills_path(&self) -> &str {
        &self.skills_path
    }

    pub fn reload_flag(&self) -> Arc<AtomicBool> {
        Arc::clone(&self.reload_flag)
    }

    pub async fn init(&self) -> Result<(), AppError> {
        self.reload().await
    }

    pub async fn check_and_reload(&self) -> Result<Option<usize>, AppError> {
        if self.reload_flag.swap(false, Ordering::SeqCst) {
            self.reload().await?;
            let count = self.tools.read().await.len();
            Ok(Some(count))
        } else {
            Ok(None)
        }
    }

    pub async fn reload(&self) -> Result<(), AppError> {
        let mut all_tools = Vec::new();
        let mut handlers = std::collections::HashMap::new();
        let file_factory = FileHandlerFactory::new(&self.skills_path);

        let configs = self.loader.load_tools()?;
        for config in &configs {
            for tool_def in &config.tools {
                let input_schema = tool_def.input_schema
                    .as_object()
                    .cloned()
                    .unwrap_or_default();

                let tool = Tool::new(
                    tool_def.name.clone(),
                    tool_def.description.clone(),
                    Arc::new(input_schema),
                );
                all_tools.push(tool);

                let handler = if tool_def.handler == "refresh_tools" {
                    let flag = Arc::clone(&self.reload_flag);
                    Some(create_refresh_handler(flag))
                } else if tool_def.handler == "file_reader" {
                    let file_path = tool_def.input_schema
                        .get("properties")
                        .and_then(|p| p.get("file_path"))
                        .and_then(|fp| fp.get("default"))
                        .and_then(|d| d.as_str())
                        .unwrap_or("");
                    Some(file_factory.create_file_handler(file_path))
                } else {
                    warn!("未知的 handler 类型: {}", tool_def.handler);
                    None
                };

                if let Some(handler) = handler {
                    handlers.insert(tool_def.name.clone(), handler);
                }
            }
        }

        let scanned_skills = self.loader.load_skills(&self.skills_path);
        
        let scanned_names: Vec<String> = scanned_skills.iter().map(|s| s.name.clone()).collect();
        
        if let Ok(all_db_skills) = self.db.list_all() {
            for db_skill in all_db_skills {
                if !scanned_names.contains(&db_skill.name) {
                    if let Err(e) = self.db.permanent_delete(&db_skill.name) {
                        warn!("删除不存在的skill记录失败: {} - {}", db_skill.name, e);
                    }
                }
            }
        }
        
        for skill in &scanned_skills {
            let tags_json = serde_json::to_string(&skill.tags).unwrap_or_else(|_| "[]".to_string());
            let dir_name = skill.path.file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("")
                .to_string();
            let skill_name = skill.name.clone();
            let skill_desc = skill.description.clone();
            let skill_file = skill.skill_file.to_string_lossy().to_string();
            let db = Arc::clone(&self.db);
            
            if let Err(e) = db.upsert(&skill_name, &skill_desc, &tags_json, &skill_file, &dir_name) {
                warn!("写入数据库失败: {}", e);
            }
        }
        
        let skills_for_tools: Vec<SkillInfo> = scanned_skills.clone();
        
        let disabled_names: Vec<String> = if let Ok(all_skills) = self.db.list_all() {
            all_skills.iter()
                .filter(|s| !s.enabled)
                .map(|s| s.name.clone())
                .collect()
        } else {
            Vec::new()
        };
        
        for skill in &skills_for_tools {
            if disabled_names.contains(&skill.name) {
                continue;
            }
            
            let input_schema = serde_json::json!({
                "type": "object",
                "properties": {
                    "file_path": {
                        "type": "string",
                        "description": "Skill文件路径",
                        "default": skill.skill_file.to_string_lossy()
                    }
                }
            });

            let tool = Tool::new(
                skill.name.clone(),
                skill.description.clone(),
                Arc::new(input_schema.as_object().cloned().unwrap_or_default()),
            );
            all_tools.push(tool);

            let handler = file_factory.create_file_handler(&skill.skill_file.to_string_lossy());
            handlers.insert(skill.name.clone(), handler);
        }

        *self.skills.write().await = scanned_skills;

        mcp_tools::register_mcp_tools(&mut all_tools, &mut handlers, &self.db, &self.skills_path);

        let refresh_flag = Arc::clone(&self.reload_flag);
        handlers.insert("refresh_skills".to_string(), create_refresh_handler(refresh_flag));

        info!("共加载 {} 个工具", all_tools.len());

        *self.tools.write().await = all_tools;
        *self.executor.write().await = ToolExecutor::from_handlers(handlers);

        Ok(())
    }

    pub async fn get_tools(&self) -> Vec<Tool> {
        self.tools.read().await.clone()
    }

    pub async fn executor(&self) -> ToolExecutor {
        self.executor.read().await.clone()
    }

    pub async fn count(&self) -> usize {
        self.tools.read().await.len()
    }
}

impl Clone for ToolManager {
    fn clone(&self) -> Self {
        Self {
            tools: Arc::clone(&self.tools),
            executor: Arc::clone(&self.executor),
            loader: self.loader.clone(),
            skills_path: self.skills_path.clone(),
            reload_flag: Arc::clone(&self.reload_flag),
            skills: Arc::clone(&self.skills),
            db: Arc::clone(&self.db),
        }
    }
}

fn create_refresh_handler(reload_flag: Arc<AtomicBool>) -> ToolHandler {
    Arc::new(move |_args| {
        let flag = Arc::clone(&reload_flag);
        Box::pin(async move {
            flag.store(true, Ordering::SeqCst);
            Ok(serde_json::json!({
                "status": "refreshing",
                "message": "工具列表将在下次调用时刷新"
            }))
        })
    })
}
