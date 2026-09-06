// Copyright (c) Mr_老鬼. All rights reserved.
// https://www.junjiestudio.top
// Derivative works must retain this copyright notice.

use std::collections::HashMap;
use std::sync::Arc;
use rmcp::model::Tool;
use tracing::info;

use crate::db::{SkillDb, SkillStore, MemoryStore, IssueStore};
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

    // Memory tools
    let remember_tool = Tool::new(
        "remember".to_string(),
        "记住一条信息（需求、方案、结果等），支持标签和来源".to_string(),
        Arc::new(serde_json::json!({
            "type": "object",
            "properties": {
                "content": { "type": "string", "description": "记忆内容" },
                "tags": { "type": "string", "description": "标签，多个用逗号分隔" },
                "source": { "type": "string", "description": "来源（如文件路径、会话ID）" }
            },
            "required": ["content"]
        }).as_object().cloned().unwrap_or_default()),
    );
    all_tools.push(remember_tool);
    let db_clone = Arc::clone(db);
    handlers.insert("remember".to_string(), create_remember_handler(db_clone));

    let recall_tool = Tool::new(
        "recall".to_string(),
        "回忆相关记忆，按关键词和标签搜索".to_string(),
        Arc::new(serde_json::json!({
            "type": "object",
            "properties": {
                "query": { "type": "string", "description": "搜索关键词" },
                "tags": { "type": "string", "description": "标签过滤，多个用逗号分隔" },
                "include_archived": { "type": "boolean", "description": "是否包含已归档记忆" },
                "limit": { "type": "integer", "description": "返回条数，默认5" }
            }
        }).as_object().cloned().unwrap_or_default()),
    );
    all_tools.push(recall_tool);
    let db_clone = Arc::clone(db);
    handlers.insert("recall".to_string(), create_recall_handler(db_clone));

    let forget_tool = Tool::new(
        "forget".to_string(),
        "永久删除一条记忆".to_string(),
        Arc::new(serde_json::json!({
            "type": "object",
            "properties": {
                "memory_id": { "type": "integer", "description": "记忆ID" }
            },
            "required": ["memory_id"]
        }).as_object().cloned().unwrap_or_default()),
    );
    all_tools.push(forget_tool);
    let db_clone = Arc::clone(db);
    handlers.insert("forget".to_string(), create_forget_handler(db_clone));

    let archive_tool = Tool::new(
        "archive_memories".to_string(),
        "归档超过指定天数的旧记忆".to_string(),
        Arc::new(serde_json::json!({
            "type": "object",
            "properties": {
                "older_than_days": { "type": "integer", "description": "归档超过指定天数的记忆，默认30" }
            }
        }).as_object().cloned().unwrap_or_default()),
    );
    all_tools.push(archive_tool);
    let db_clone = Arc::clone(db);
    handlers.insert("archive_memories".to_string(), create_archive_handler(db_clone));

    let stats_tool = Tool::new(
        "get_memory_stats".to_string(),
        "获取记忆统计信息".to_string(),
        Arc::new(serde_json::json!({
            "type": "object",
            "properties": {}
        }).as_object().cloned().unwrap_or_default()),
    );
    all_tools.push(stats_tool);
    let db_clone = Arc::clone(db);
    handlers.insert("get_memory_stats".to_string(), create_stats_handler(db_clone));

    // Issue tools
    let search_issues_tool = Tool::new(
        "search_issues".to_string(),
        "搜索常见问题库，按场景和解决方案查找".to_string(),
        Arc::new(serde_json::json!({
            "type": "object",
            "properties": {
                "query": { "type": "string", "description": "搜索关键词" },
                "task": { "type": "string", "description": "任务场景过滤" },
                "category": { "type": "string", "description": "分类过滤" },
                "limit": { "type": "integer", "description": "返回条数，默认10" }
            }
        }).as_object().cloned().unwrap_or_default()),
    );
    all_tools.push(search_issues_tool);
    let db_clone = Arc::clone(db);
    handlers.insert("search_issues".to_string(), create_search_issues_handler(db_clone));

    let report_issue_tool = Tool::new(
        "report_issue".to_string(),
        "报告新问题，积累问题库".to_string(),
        Arc::new(serde_json::json!({
            "type": "object",
            "properties": {
                "category": { "type": "string", "description": "分类" },
                "task": { "type": "string", "description": "任务场景" },
                "scenario": { "type": "string", "description": "问题现象" },
                "solution": { "type": "string", "description": "解决方案" },
                "related_api": { "type": "string", "description": "相关API" }
            },
            "required": ["category", "scenario", "solution"]
        }).as_object().cloned().unwrap_or_default()),
    );
    all_tools.push(report_issue_tool);
    let db_clone = Arc::clone(db);
    handlers.insert("report_issue".to_string(), create_report_issue_handler(db_clone));

    info!("注册了 7 个记忆/问题 MCP 工具");
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

fn create_remember_handler(db: Arc<SkillDb>) -> ToolHandler {
    Arc::new(move |args| {
        let db = Arc::clone(&db);
        Box::pin(async move {
            let content = args.get("content").and_then(|v| v.as_str()).unwrap_or("");
            let tags = args.get("tags").and_then(|v| v.as_str());
            let source = args.get("source").and_then(|v| v.as_str());
            
            if content.is_empty() {
                return Ok(serde_json::json!({ "error": "请提供 content 参数" }));
            }
            
            match db.remember(content, tags, source) {
                Ok(id) => Ok(serde_json::json!({
                    "status": "success",
                    "memory_id": id,
                    "message": format!("已记住，ID: {}", id)
                })),
                Err(e) => Ok(serde_json::json!({ "error": format!("记住失败: {}", e) })),
            }
        })
    })
}

fn create_recall_handler(db: Arc<SkillDb>) -> ToolHandler {
    Arc::new(move |args| {
        let db = Arc::clone(&db);
        Box::pin(async move {
            let query = args.get("query").and_then(|v| v.as_str()).unwrap_or("");
            let tags = args.get("tags").and_then(|v| v.as_str())
                .map(|t| t.split(',').map(|s| s.trim().to_string()).collect());
            let include_archived = args.get("include_archived").and_then(|v| v.as_bool()).unwrap_or(false);
            let limit = args.get("limit").and_then(|v| v.as_i64()).unwrap_or(5);
            
            if query.is_empty() && tags.is_none() {
                return Ok(serde_json::json!({ "error": "请提供 query 或 tags 参数" }));
            }
            
            match db.recall(query, tags, include_archived, limit) {
                Ok(memories) => {
                    let items: Vec<serde_json::Value> = memories.iter().map(|m| {
                        serde_json::json!({
                            "id": m.id,
                            "content": m.content,
                            "tags": m.tags,
                            "source": m.source,
                            "created_at": m.created_at
                        })
                    }).collect();
                    Ok(serde_json::json!({
                        "count": items.len(),
                        "memories": items
                    }))
                },
                Err(e) => Ok(serde_json::json!({ "error": format!("搜索失败: {}", e) })),
            }
        })
    })
}

fn create_forget_handler(db: Arc<SkillDb>) -> ToolHandler {
    Arc::new(move |args| {
        let db = Arc::clone(&db);
        Box::pin(async move {
            let memory_id = args.get("memory_id").and_then(|v| v.as_i64()).unwrap_or(0);
            if memory_id == 0 {
                return Ok(serde_json::json!({ "error": "请提供 memory_id 参数" }));
            }
            match db.forget(memory_id) {
                Ok(()) => Ok(serde_json::json!({
                    "status": "success",
                    "message": format!("已删除记忆 ID: {}", memory_id)
                })),
                Err(e) => Ok(serde_json::json!({ "error": format!("删除失败: {}", e) })),
            }
        })
    })
}

fn create_archive_handler(db: Arc<SkillDb>) -> ToolHandler {
    Arc::new(move |args| {
        let db = Arc::clone(&db);
        Box::pin(async move {
            let days = args.get("older_than_days").and_then(|v| v.as_i64()).unwrap_or(30);
            match db.archive(days) {
                Ok(count) => Ok(serde_json::json!({
                    "status": "success",
                    "archived_count": count,
                    "message": format!("已归档 {} 条记忆", count)
                })),
                Err(e) => Ok(serde_json::json!({ "error": format!("归档失败: {}", e) })),
            }
        })
    })
}

fn create_stats_handler(db: Arc<SkillDb>) -> ToolHandler {
    Arc::new(move |_args| {
        let db = Arc::clone(&db);
        Box::pin(async move {
            match db.get_stats() {
                Ok(stats) => Ok(serde_json::json!({
                    "total": stats.total,
                    "active": stats.active,
                    "archived": stats.archived,
                    "archive_rate": format!("{:.2}%", stats.archive_rate * 100.0)
                })),
                Err(e) => Ok(serde_json::json!({ "error": format!("获取统计失败: {}", e) })),
            }
        })
    })
}

fn create_search_issues_handler(db: Arc<SkillDb>) -> ToolHandler {
    Arc::new(move |args| {
        let db = Arc::clone(&db);
        Box::pin(async move {
            let query = args.get("query").and_then(|v| v.as_str()).unwrap_or("");
            let task = args.get("task").and_then(|v| v.as_str());
            let category = args.get("category").and_then(|v| v.as_str());
            let limit = args.get("limit").and_then(|v| v.as_i64()).unwrap_or(10);
            
            if query.is_empty() && task.is_none() && category.is_none() {
                return Ok(serde_json::json!({ "error": "请提供 query、task 或 category 参数" }));
            }
            
            match db.search_issues(query, task, category, limit) {
                Ok(issues) => {
                    let items: Vec<serde_json::Value> = issues.iter().map(|i| {
                        serde_json::json!({
                            "id": i.id,
                            "issue_id": i.issue_id,
                            "category": i.category,
                            "task": i.task,
                            "scenario": i.scenario,
                            "solution": i.solution,
                            "related_api": i.related_api,
                            "frequency": i.frequency
                        })
                    }).collect();
                    Ok(serde_json::json!({
                        "count": items.len(),
                        "issues": items
                    }))
                },
                Err(e) => Ok(serde_json::json!({ "error": format!("搜索失败: {}", e) })),
            }
        })
    })
}

fn create_report_issue_handler(db: Arc<SkillDb>) -> ToolHandler {
    Arc::new(move |args| {
        let db = Arc::clone(&db);
        Box::pin(async move {
            let category = args.get("category").and_then(|v| v.as_str()).unwrap_or("");
            let task = args.get("task").and_then(|v| v.as_str());
            let scenario = args.get("scenario").and_then(|v| v.as_str()).unwrap_or("");
            let solution = args.get("solution").and_then(|v| v.as_str()).unwrap_or("");
            let related_api = args.get("related_api").and_then(|v| v.as_str());
            
            if category.is_empty() || scenario.is_empty() || solution.is_empty() {
                return Ok(serde_json::json!({ "error": "请提供 category、scenario、solution 参数" }));
            }
            
            match db.report_issue(category, task, scenario, solution, related_api) {
                Ok(id) => Ok(serde_json::json!({
                    "status": "success",
                    "issue_id": id,
                    "message": format!("已记录问题，ID: {}", id)
                })),
                Err(e) => Ok(serde_json::json!({ "error": format!("记录失败: {}", e) })),
            }
        })
    })
}
