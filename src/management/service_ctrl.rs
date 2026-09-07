// Copyright (c) Mr_老鬼. All rights reserved.
// https://www.junjiestudio.top
// Derivative works must retain this copyright notice.

use std::process::Command;
use std::os::windows::process::CommandExt;
use tracing::{info, error};

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

pub fn is_port_in_use(addr: &str) -> bool {
    use std::net::TcpListener;
    TcpListener::bind(addr).is_err()
}

pub fn start_service(exe_dir: &std::path::Path) -> Result<String, String> {
    info!("尝试启动 MCP 服务，exe_dir: {:?}", exe_dir);
    
    if check_service_running() {
        info!("MCP 服务已经在运行");
        return Ok("MCP 服务已经在运行".to_string());
    }
    
    let exe_path = exe_dir.join("AISkillBox-mcp.exe");
    info!("MCP 服务程序路径: {:?}", exe_path);
    
    if !exe_path.exists() {
        let msg = format!("MCP 服务程序不存在: {:?}", exe_path);
        error!("{}", msg);
        return Err(msg);
    }
    
    // 检查端口是否被占用
    if is_port_in_use("127.0.0.1:10881") {
        info!("端口 10881 被占用，等待释放...");
        std::thread::sleep(std::time::Duration::from_millis(1000));
    }
    
    match Command::new(&exe_path)
        .current_dir(exe_dir)
        .creation_flags(0x08000000)
        .spawn() {
        Ok(_) => {
            info!("MCP 服务已启动");
            // 等待服务真正启动
            std::thread::sleep(std::time::Duration::from_millis(500));
            if check_service_running() {
                Ok("MCP 服务已启动".to_string())
            } else {
                Err("MCP 服务启动后未检测到进程".to_string())
            }
        }
        Err(e) => {
            let msg = format!("启动 MCP 服务失败: {}", e);
            error!("{}", msg);
            Err(msg)
        }
    }
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
    
    // 等待端口释放
    for _ in 0..20 {
        std::thread::sleep(std::time::Duration::from_millis(200));
        if !check_service_running() {
            break;
        }
    }
    
    // 额外等待确保端口完全释放
    std::thread::sleep(std::time::Duration::from_millis(500));
    
    start_service(exe_dir)
}
