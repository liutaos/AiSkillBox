// Copyright (c) Mr_老鬼. All rights reserved.
// https://www.junjiestudio.top
// Derivative works must retain this copyright notice.

use salvo::prelude::*;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;

use crate::db::{SkillDb, SkillStore};
use crate::management::service_ctrl;
use crate::tools::skill_scanner::parse_front_matter;

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

#[handler]
pub async fn restart_service(depot: &mut Depot, res: &mut Response) {
    let exe_dir = depot.get_typed::<PathBuf>().unwrap();
    
    tracing::info!("收到重启请求，开始重启 MCP 服务...");
    
    let exe_path = exe_dir.join("AISkillBox-mcp.exe");
    
    // PowerShell 完全静默执行，无任何窗口
    let ps_script = format!(
        "Start-Sleep -Seconds 3; \
         Stop-Process -Name AISkillBox-mcp -Force -ErrorAction SilentlyContinue; \
         Start-Sleep -Seconds 2; \
         Start-Process -FilePath '{exe}' -WindowStyle Hidden",
        exe = exe_path.to_string_lossy()
    );
    
    use std::os::windows::process::CommandExt;
    let result = std::process::Command::new("powershell")
        .args(["-WindowStyle", "Hidden", "-Command", &ps_script])
        .creation_flags(0x08000000)
        .spawn();
    
    match result {
        Ok(_) => {
            tracing::info!("重启守护已启动");
            res.render(Json(serde_json::json!({
                "success": true,
                "message": "正在重启服务..."
            })));
        }
        Err(e) => {
            tracing::error!("启动重启守护失败: {}", e);
            res.render(Json(serde_json::json!({
                "success": false,
                "message": format!("启动重启脚本失败: {}", e)
            })));
        }
    }
}

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

#[handler]
pub async fn refresh_skills(depot: &mut Depot, res: &mut Response) {
    let db = depot.get_typed::<Arc<SkillDb>>().unwrap();
    let skills_dir = depot.get_typed::<String>().unwrap();
    let reload_flag = depot.get_typed::<Arc<AtomicBool>>().unwrap();
    
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
    
    if let Ok(all_db_skills) = db.list_all() {
        for db_skill in all_db_skills {
            if !scanned_names.contains(&db_skill.name) {
                let _ = db.permanent_delete(&db_skill.name);
            }
        }
    }
    
    reload_flag.store(true, std::sync::atomic::Ordering::SeqCst);
    
    res.render(Json(serde_json::json!({
        "success": true,
        "message": format!("已扫描并更新 {} 个skill", count)
    })));
}
