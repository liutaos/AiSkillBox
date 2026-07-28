use std::collections::HashMap;
use std::sync::Arc;
use rmcp::model::Tool;
use tracing::info;

use crate::db::{SkillDb, SkillStore};
use crate::exec::ToolHandler;

pub fn register_mcp_tools(
    all_tools: &mut Vec<Tool>,
    handlers: &mut HashMap<String, ToolHandler>,
    db: &Arc<SkillDb>,
    skills_path: &str,
) {
    let refresh_tool = Tool::new(
        "refresh_skills".to_string(),
        "刷新skill列表，重新扫描skills目录".to_string(),
        Arc::new(serde_json::json!({
            "type": "object",
            "properties": {}
        }).as_object().cloned().unwrap_or_default()),
    );
    all_tools.push(refresh_tool);

    let delete_tool = Tool::new(
        "delete_skill".to_string(),
        "删除指定的skill（移动到回收站）".to_string(),
        Arc::new(serde_json::json!({
            "type": "object",
            "properties": {
                "skill_name": {
                    "type": "string",
                    "description": "要删除的skill名称"
                }
            },
            "required": ["skill_name"]
        }).as_object().cloned().unwrap_or_default()),
    );
    all_tools.push(delete_tool);
    let skills_path_clone = skills_path.to_string();
    let db_clone = Arc::clone(db);
    handlers.insert("delete_skill".to_string(), create_delete_handler(skills_path_clone, db_clone));

    let restore_tool = Tool::new(
        "restore_skill".to_string(),
        "从回收站恢复指定的skill".to_string(),
        Arc::new(serde_json::json!({
            "type": "object",
            "properties": {
                "skill_name": {
                    "type": "string",
                    "description": "要恢复的skill名称"
                }
            },
            "required": ["skill_name"]
        }).as_object().cloned().unwrap_or_default()),
    );
    all_tools.push(restore_tool);
    let skills_path_clone = skills_path.to_string();
    let db_clone = Arc::clone(db);
    handlers.insert("restore_skill".to_string(), create_restore_handler(skills_path_clone, db_clone));

    let search_tool = Tool::new(
        "search_skills".to_string(),
        "搜索skill，支持关键词和标签过滤".to_string(),
        Arc::new(serde_json::json!({
            "type": "object",
            "properties": {
                "query": {
                    "type": "string",
                    "description": "搜索关键词"
                },
                "tags": {
                    "type": "string",
                    "description": "标签过滤，多个用逗号分隔"
                }
            },
            "required": ["query"]
        }).as_object().cloned().unwrap_or_default()),
    );
    all_tools.push(search_tool);
    let db_clone = Arc::clone(db);
    handlers.insert("search_skills".to_string(), create_search_handler(db_clone));

    let list_tool = Tool::new(
        "list_skills".to_string(),
        "列出所有skill，支持标签过滤".to_string(),
        Arc::new(serde_json::json!({
            "type": "object",
            "properties": {
                "tags": {
                    "type": "string",
                    "description": "标签过滤，多个用逗号分隔"
                }
            }
        }).as_object().cloned().unwrap_or_default()),
    );
    all_tools.push(list_tool);
    let db_clone = Arc::clone(db);
    handlers.insert("list_skills".to_string(), create_list_handler(db_clone));

    let trash_tool = Tool::new(
        "list_trash".to_string(),
        "列出回收站中的skill（已删除的skill）".to_string(),
        Arc::new(serde_json::json!({
            "type": "object",
            "properties": {}
        }).as_object().cloned().unwrap_or_default()),
    );
    all_tools.push(trash_tool);
    let db_clone = Arc::clone(db);
    handlers.insert("list_trash".to_string(), create_trash_handler(db_clone));

    let enable_tool = Tool::new(
        "enable_skill".to_string(),
        "启用指定的skill".to_string(),
        Arc::new(serde_json::json!({
            "type": "object",
            "properties": {
                "skill_name": {
                    "type": "string",
                    "description": "要启用的skill名称"
                }
            },
            "required": ["skill_name"]
        }).as_object().cloned().unwrap_or_default()),
    );
    all_tools.push(enable_tool);
    let db_clone = Arc::clone(db);
    handlers.insert("enable_skill".to_string(), create_enable_handler(db_clone));

    let disable_tool = Tool::new(
        "disable_skill".to_string(),
        "禁用指定的skill".to_string(),
        Arc::new(serde_json::json!({
            "type": "object",
            "properties": {
                "skill_name": {
                    "type": "string",
                    "description": "要禁用的skill名称"
                }
            },
            "required": ["skill_name"]
        }).as_object().cloned().unwrap_or_default()),
    );
    all_tools.push(disable_tool);
    let db_clone = Arc::clone(db);
    handlers.insert("disable_skill".to_string(), create_disable_handler(db_clone));

    let migrate_tool = Tool::new(
        "migrate_skills".to_string(),
        "获取MCP托管目录路径及常见skill存放位置，用于迁移".to_string(),
        Arc::new(serde_json::json!({
            "type": "object",
            "properties": {}
        }).as_object().cloned().unwrap_or_default()),
    );
    all_tools.push(migrate_tool);
    let skills_path_clone = skills_path.to_string();
    let db_clone = Arc::clone(db);
    handlers.insert("migrate_skills".to_string(), create_migrate_handler(skills_path_clone, db_clone));

    info!("注册了 7 个 MCP 内置工具");
}

fn create_delete_handler(skills_path: String, db: Arc<SkillDb>) -> ToolHandler {
    Arc::new(move |args| {
        let skills_path = skills_path.clone();
        let db = Arc::clone(&db);
        Box::pin(async move {
            let skill_name = args.get("skill_name")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            
            if skill_name.is_empty() {
                return Ok(serde_json::json!({
                    "error": "请提供 skill_name 参数"
                }));
            }
            
            let skills_dir = std::path::Path::new(&skills_path);
            let skill_dir = skills_dir.join(skill_name);
            
            if !skill_dir.exists() {
                return Ok(serde_json::json!({
                    "error": format!("Skill '{}' 不存在", skill_name)
                }));
            }
            
            let trash_dir = skills_dir.parent()
                .unwrap_or(skills_dir)
                .join("skill-trash");
            if let Err(e) = std::fs::create_dir_all(&trash_dir) {
                return Ok(serde_json::json!({
                    "error": format!("创建回收站失败: {}", e)
                }));
            }
            
            let timestamp = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs();
            let trash_path = trash_dir.join(format!("{}_{}", skill_name, timestamp));
            
            if let Err(e) = std::fs::rename(&skill_dir, &trash_path) {
                return Ok(serde_json::json!({
                    "error": format!("移动到回收站失败: {}", e)
                }));
            }
            
            if let Err(e) = db.soft_delete(skill_name) {
                return Ok(serde_json::json!({
                    "error": format!("目录已移动，但数据库更新失败: {}", e)
                }));
            }
            
            Ok(serde_json::json!({
                "status": "success",
                "message": format!("Skill '{}' 已删除，移至回收站", skill_name)
            }))
        })
    })
}

fn create_restore_handler(skills_path: String, db: Arc<SkillDb>) -> ToolHandler {
    Arc::new(move |args| {
        let skills_path = skills_path.clone();
        let db = Arc::clone(&db);
        Box::pin(async move {
            let skill_name = args.get("skill_name")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            
            if skill_name.is_empty() {
                return Ok(serde_json::json!({
                    "error": "请提供 skill_name 参数"
                }));
            }
            
            let skills_dir = std::path::Path::new(&skills_path);
            let trash_dir = skills_dir.parent()
                .unwrap_or(skills_dir)
                .join("skill-trash");
            
            if !trash_dir.exists() {
                return Ok(serde_json::json!({
                    "error": "回收站目录不存在"
                }));
            }
            
            let mut found_path = None;
            if let Ok(entries) = std::fs::read_dir(&trash_dir) {
                for entry in entries.flatten() {
                    let name = entry.file_name().to_string_lossy().to_string();
                    if (name == skill_name || name.starts_with(&format!("{}_{}", skill_name, ""))) && entry.path().is_dir() {
                        found_path = Some(entry.path());
                        break;
                    }
                }
            }
            
            let trash_path = match found_path {
                Some(p) => p,
                None => {
                    return Ok(serde_json::json!({
                        "error": format!("在回收站中未找到 skill '{}'", skill_name)
                    }));
                }
            };
            
            let restore_path = skills_dir.join(skill_name);
            if let Err(e) = std::fs::rename(&trash_path, &restore_path) {
                return Ok(serde_json::json!({
                    "error": format!("恢复失败: {}", e)
                }));
            }
            
            if let Err(e) = db.restore(skill_name) {
                return Ok(serde_json::json!({
                    "error": format!("目录已恢复，但数据库更新失败: {}", e)
                }));
            }
            
            Ok(serde_json::json!({
                "status": "success",
                "message": format!("Skill '{}' 已从回收站恢复", skill_name)
            }))
        })
    })
}

fn create_search_handler(db: Arc<SkillDb>) -> ToolHandler {
    Arc::new(move |args| {
        let db = Arc::clone(&db);
        Box::pin(async move {
            let query = args.get("query")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            
            let tags = args.get("tags")
                .and_then(|v| v.as_str());
            
            if query.is_empty() {
                return Ok(serde_json::json!({
                    "error": "请提供 query 参数"
                }));
            }
            
            let results = db.search(query, tags).unwrap_or_default();
            
            let enabled_count = results.iter().filter(|s| s.enabled).count();
            let disabled_count = results.iter().filter(|s| !s.enabled).count();
            
            let skills: Vec<serde_json::Value> = results.iter().map(|s| {
                serde_json::json!({
                    "name": s.name,
                    "description": s.description,
                    "tags": s.tags,
                    "dir": s.dir_name,
                    "enabled": s.enabled
                })
            }).collect();
            
            Ok(serde_json::json!({
                "count": skills.len(),
                "enabled_count": enabled_count,
                "disabled_count": disabled_count,
                "skills": skills
            }))
        })
    })
}

fn create_list_handler(db: Arc<SkillDb>) -> ToolHandler {
    Arc::new(move |args| {
        let db = Arc::clone(&db);
        Box::pin(async move {
            let tags = args.get("tags")
                .and_then(|v| v.as_str());
            
            let results = if let Some(tags) = tags {
                db.search("", Some(tags)).unwrap_or_default()
            } else {
                db.list_all().unwrap_or_default()
            };
            
            let enabled_count = results.iter().filter(|s| s.enabled).count();
            let disabled_count = results.iter().filter(|s| !s.enabled).count();
            
            let skills: Vec<serde_json::Value> = results.iter().map(|s| {
                serde_json::json!({
                    "name": s.name,
                    "description": s.description,
                    "tags": s.tags,
                    "dir": s.dir_name,
                    "enabled": s.enabled
                })
            }).collect();
            
            Ok(serde_json::json!({
                "count": skills.len(),
                "enabled_count": enabled_count,
                "disabled_count": disabled_count,
                "skills": skills
            }))
        })
    })
}

fn create_trash_handler(db: Arc<SkillDb>) -> ToolHandler {
    Arc::new(move |_args| {
        let db = Arc::clone(&db);
        Box::pin(async move {
            let results = db.list_trash().unwrap_or_default();
            
            let skills: Vec<serde_json::Value> = results.iter().map(|s| {
                serde_json::json!({
                    "name": s.name,
                    "description": s.description,
                    "tags": s.tags,
                    "dir": s.dir_name,
                    "deleted_at": s.updated_at
                })
            }).collect();
            
            Ok(serde_json::json!({
                "count": skills.len(),
                "skills": skills
            }))
        })
    })
}

fn create_enable_handler(db: Arc<SkillDb>) -> ToolHandler {
    Arc::new(move |args| {
        let db = Arc::clone(&db);
        Box::pin(async move {
            let skill_name = args.get("skill_name")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            
            if skill_name.is_empty() {
                return Ok(serde_json::json!({
                    "error": "请提供 skill_name 参数"
                }));
            }
            
            if let Ok(Some(skill)) = db.get_by_name(skill_name) {
                if skill.deleted {
                    return Ok(serde_json::json!({
                        "error": format!("Skill '{}' 已在回收站，请先使用 restore_skill 恢复", skill_name)
                    }));
                }
            }
            
            match db.set_enabled(skill_name, true) {
                Ok(()) => Ok(serde_json::json!({
                    "status": "success",
                    "message": format!("Skill '{}' 已启用", skill_name)
                })),
                Err(e) => Ok(serde_json::json!({
                    "error": format!("启用失败: {}", e)
                })),
            }
        })
    })
}

fn create_disable_handler(db: Arc<SkillDb>) -> ToolHandler {
    Arc::new(move |args| {
        let db = Arc::clone(&db);
        Box::pin(async move {
            let skill_name = args.get("skill_name")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            
            if skill_name.is_empty() {
                return Ok(serde_json::json!({
                    "error": "请提供 skill_name 参数"
                }));
            }
            
            if let Ok(Some(skill)) = db.get_by_name(skill_name) {
                if skill.deleted {
                    return Ok(serde_json::json!({
                        "error": format!("Skill '{}' 已在回收站，请先使用 restore_skill 恢复", skill_name)
                    }));
                }
            }
            
            match db.set_enabled(skill_name, false) {
                Ok(()) => Ok(serde_json::json!({
                    "status": "success",
                    "message": format!("Skill '{}' 已禁用", skill_name)
                })),
                Err(e) => Ok(serde_json::json!({
                    "error": format!("禁用失败: {}", e)
                })),
            }
        })
    })
}

fn create_migrate_handler(skills_path: String, _db: Arc<SkillDb>) -> ToolHandler {
    Arc::new(move |_args| {
        let skills_path = skills_path.clone();
        Box::pin(async move {
            let home = std::env::var("USERPROFILE")
                .or_else(|_| std::env::var("HOME"))
                .unwrap_or_default();
            
            Ok(serde_json::json!({
                "managed_dir": skills_path,
                "common_locations": [
                    format!("{}/.agents/skills", home),
                    format!("{}/.cursor/skills", home),
                    format!("{}/.windsurf/skills", home),
                    format!("{}/.copilot/skills", home),
                    format!("{}/.config/opencode/skills", home),
                    format!("{}/.trae/skills", home),
                    format!("{}/.qoder/skills", home),
                    format!("{}/.workbuddy/skills", home),
                    format!("{}/.codebuddy/skills", home)
                ],
                "rules": [
                    "扫描上述目录，找到含SKILL.md的目录即为skill",
                    "已存在于managed_dir的skill跳过，告知用户",
                    "重名冲突必须询问用户，用户确认后再移动",
                    "移动完成后调用 refresh_skills 刷新"
                ]
            }))
        })
    })
}
