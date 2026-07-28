// Copyright (c) Mr_老鬼. All rights reserved.
// https://www.junjiestudio.top

use std::process::Command;
use std::os::windows::process::CommandExt;
use tracing::info;

pub fn check_service_running() -> bool {
    let output = Command::new("tasklist")
        .args(["/FI", "IMAGENAME eq AISkillBox-mcp.exe", "/NH"])
        .creation_flags(0x08000000)
        .output();
    
    match output {
        Ok(out) => {
            let stdout = String::from_utf8_lossy(&out.stdout);
            stdout.contains("AISkillBox-mcp.exe")
        }
        Err(_) => false,
    }
}

pub fn start_service(exe_dir: &std::path::Path) -> Result<String, String> {
    if check_service_running() {
        return Ok("MCP 服务已经在运行".to_string());
    }
    
    let exe_path = exe_dir.join("AISkillBox-mcp.exe");
    if !exe_path.exists() {
        return Err(format!("MCP 服务程序不存在: {:?}", exe_path));
    }
    
    Command::new(&exe_path)
        .current_dir(exe_dir)
        .creation_flags(0x08000000)
        .spawn()
        .map_err(|e| format!("启动 MCP 服务失败: {}", e))?;
    
    info!("MCP 服务已启动");
    Ok("MCP 服务已启动".to_string())
}

pub fn stop_service() -> Result<String, String> {
    if !check_service_running() {
        return Ok("MCP 服务未运行".to_string());
    }
    
    Command::new("taskkill")
        .args(["/F", "/IM", "AISkillBox-mcp.exe"])
        .creation_flags(0x08000000)
        .output()
        .map_err(|e| format!("停止 MCP 服务失败: {}", e))?;
    
    info!("MCP 服务已停止");
    Ok("MCP 服务已停止".to_string())
}

pub fn restart_service(exe_dir: &std::path::Path) -> Result<String, String> {
    let _ = stop_service();
    std::thread::sleep(std::time::Duration::from_millis(500));
    start_service(exe_dir)
}
