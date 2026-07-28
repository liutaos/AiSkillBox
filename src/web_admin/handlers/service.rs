use salvo::prelude::*;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;

use crate::db::{SkillDb, SkillStore};
use crate::management::service_ctrl;

/// 启动 MCP 服务
#[handler]
pub async fn start_service(depot: &mut Depot, res: &mut Response) {
    let exe_dir = depot.get_typed::<PathBuf>().unwrap();
    
    match service_ctrl::start_service(exe_dir) {
        Ok(msg) => {
            res.render(Json(serde_json::json!({
                "success": true,
                "message": msg
            })));
        }
        Err(e) => {
            res.status_code(StatusCode::INTERNAL_SERVER_ERROR);
            res.render(Json(serde_json::json!({
                "success": false,
                "message": e
            })));
        }
    }
}

/// 停止 MCP 服务
#[handler]
pub async fn stop_service(_req: &mut Request, res: &mut Response) {
    match service_ctrl::stop_service() {
        Ok(msg) => {
            res.render(Json(serde_json::json!({
                "success": true,
                "message": msg
            })));
        }
        Err(e) => {
            res.status_code(StatusCode::INTERNAL_SERVER_ERROR);
            res.render(Json(serde_json::json!({
                "success": false,
                "message": e
            })));
        }
    }
}

/// 重启 MCP 服务
#[handler]
pub async fn restart_service(depot: &mut Depot, res: &mut Response) {
    let exe_dir = depot.get_typed::<PathBuf>().unwrap();
    
    match service_ctrl::restart_service(exe_dir) {
        Ok(msg) => {
            res.render(Json(serde_json::json!({
                "success": true,
                "message": msg
            })));
        }
        Err(e) => {
            res.status_code(StatusCode::INTERNAL_SERVER_ERROR);
            res.render(Json(serde_json::json!({
                "success": false,
                "message": e
            })));
        }
    }
}

/// 检查服务状态
#[handler]
pub async fn check_status(_req: &mut Request, res: &mut Response) {
    let running = service_ctrl::check_service_running();
    res.render(Json(serde_json::json!({
        "success": true,
        "data": {
            "running": running
        }
    })));
}

/// 刷新 skill 列表（立即扫描目录并更新数据库）
#[handler]
pub async fn refresh_skills(depot: &mut Depot, res: &mut Response) {
    let db = depot.get_typed::<Arc<SkillDb>>().unwrap();
    let skills_dir = depot.get_typed::<String>().unwrap();
    let reload_flag = depot.get_typed::<Arc<AtomicBool>>().unwrap();
    
    // 扫描目录
    let skills_path = std::path::Path::new(skills_dir);
    let mut count = 0;
    let mut scanned_names = Vec::new();
    
    if let Ok(read_dir) = std::fs::read_dir(skills_path) {
        for entry in read_dir.flatten() {
            let path = entry.path();
            if !path.is_dir() {
                continue;
            }
            let skill_file = path.join("SKILL.md");
            if !skill_file.exists() {
                continue;
            }
            if let Ok(content) = std::fs::read_to_string(&skill_file) {
                let (name, description, tags) = parse_front_matter(&content);
                let dir_name = path.file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("")
                    .to_string();
                let skill_name = if name.is_empty() { dir_name.clone() } else { name };
                let file_path = skill_file.to_string_lossy().to_string();
                let tags_json = if tags.is_empty() { "[]".to_string() } else { tags };
                
                scanned_names.push(skill_name.clone());
                let _ = db.upsert(&skill_name, &description, &tags_json, &file_path, &dir_name);
                count += 1;
            }
        }
    }
    
    // 同步数据库：删除目录中不存在的skill
    if let Ok(all_db_skills) = db.list_all() {
        for db_skill in all_db_skills {
            if !scanned_names.contains(&db_skill.name) {
                let _ = db.permanent_delete(&db_skill.name);
            }
        }
    }
    
    // 设置 MCP 服务刷新标记
    reload_flag.store(true, std::sync::atomic::Ordering::SeqCst);
    
    res.render(Json(serde_json::json!({
        "success": true,
        "message": format!("已扫描并更新 {} 个skill", count)
    })));
}

/// 解析 YAML front matter
fn parse_front_matter(content: &str) -> (String, String, String) {
    let mut name = String::new();
    let mut description = String::new();
    let mut tags = String::new();

    if let Some(fm_start) = content.find("---") {
        let rest = &content[fm_start + 3..];
        if let Some(fm_end) = rest.find("---") {
            let yaml_str = &rest[..fm_end];
            if let Ok(docs) = yaml_rust2::YamlLoader::load_from_str(yaml_str) {
                if let Some(doc) = docs.first() {
                    if let Some(n) = doc["name"].as_str() {
                        name = n.to_string();
                    }
                    if let Some(d) = doc["description"].as_str() {
                        description = d.to_string();
                    }
                    if let Some(t) = doc["tags"].as_vec() {
                        let tag_list: Vec<String> = t.iter()
                            .filter_map(|tag| tag.as_str().map(|s| s.to_string()))
                            .collect();
                        tags = serde_json::to_string(&tag_list).unwrap_or_default();
                    }
                }
            }
        }
    }
    (name, description, tags)
}
