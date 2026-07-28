// Copyright (c) Mr_老鬼. All rights reserved.
// https://www.junjiestudio.top

use std::path::{Path, PathBuf};

pub fn read_config_port(exe_dir: &Path) -> String {
    let config_path = exe_dir.join("config.toml");
    if let Ok(content) = std::fs::read_to_string(&config_path) {
        for line in content.lines() {
            let line = line.trim();
            if line.starts_with("mcp_listen_addr") {
                if let Some(addr) = line.split('=').nth(1) {
                    let addr = addr.trim().trim_matches('"');
                    if let Some(port) = addr.rsplit(':').next() {
                        return port.to_string();
                    }
                }
            }
        }
    }
    "10881".to_string()
}

pub fn read_config_web_port(exe_dir: &Path) -> String {
    let config_path = exe_dir.join("config.toml");
    if let Ok(content) = std::fs::read_to_string(&config_path) {
        for line in content.lines() {
            let line = line.trim();
            if line.starts_with("listen_addr") && !line.starts_with("mcp_listen_addr") {
                if let Some(addr) = line.split('=').nth(1) {
                    let addr = addr.trim().trim_matches('"');
                    if let Some(port) = addr.rsplit(':').next() {
                        return port.to_string();
                    }
                }
            }
        }
    }
    "10882".to_string()
}

pub fn read_config_listen_host(exe_dir: &Path) -> String {
    let config_path = exe_dir.join("config.toml");
    if let Ok(content) = std::fs::read_to_string(&config_path) {
        for line in content.lines() {
            let line = line.trim();
            if line.starts_with("listen_addr") && !line.starts_with("mcp_listen_addr") {
                if let Some(addr) = line.split('=').nth(1) {
                    let addr = addr.trim().trim_matches('"');
                    if let Some(host) = addr.split(':').next() {
                        return host.to_string();
                    }
                }
            }
        }
    }
    "127.0.0.1".to_string()
}

pub fn save_port_to_config(port: &str, web_port: &str) {
    let exe_dir = get_exe_dir();
    let host = read_config_listen_host(&exe_dir);
    let config_path = exe_dir.join("config.toml");
    let content = format!(
        "listen_addr = \"{}:{}\"\nmcp_listen_addr = \"{}:{}\"\ndb_path = \"\"\n\n[mcp]\ntools_path = \"tools\"\nskills_dir = \"skills\"\n\n[log]\nlevel = \"info\"\n",
        host, web_port, host, port
    );
    std::fs::write(config_path, content).ok();
}

pub fn read_config_use_tray(exe_dir: &Path) -> bool {
    let config_path = exe_dir.join("config.toml");
    if let Ok(content) = std::fs::read_to_string(&config_path) {
        for line in content.lines() {
            let line = line.trim();
            if line.starts_with("use_tray") {
                return line.contains("true");
            }
        }
    }
    false
}

pub fn save_config_use_tray(_exe_dir: &Path, value: bool) {
    let exe_dir = get_exe_dir();
    let host = read_config_listen_host(&exe_dir);
    let dark = read_config_dark_mode_override(&exe_dir);
    let config_path = exe_dir.join("config.toml");
    let dark_str = match dark {
        Some(true) => "dark",
        Some(false) => "light",
        None => "system",
    };
    let content = format!(
        "listen_addr = \"{}:10882\"\nmcp_listen_addr = \"{}:10881\"\ndb_path = \"\"\n\n[mcp]\ntools_path = \"tools\"\nskills_dir = \"skills\"\n\n[log]\nlevel = \"info\"\n\n[gui]\nuse_tray = {}\ndark_mode = \"{}\"\n",
        host, host, value, dark_str
    );
    std::fs::write(config_path, content).ok();
}

pub fn read_config_dark_mode_override(exe_dir: &Path) -> Option<bool> {
    let config_path = exe_dir.join("config.toml");
    if let Ok(content) = std::fs::read_to_string(&config_path) {
        for line in content.lines() {
            let line = line.trim();
            if line.starts_with("dark_mode") {
                if let Some(val) = line.split('=').nth(1) {
                    let val = val.trim().trim_matches('"');
                    return match val {
                        "dark" => Some(true),
                        "light" => Some(false),
                        _ => None,
                    };
                }
            }
        }
    }
    None
}

pub fn save_config_dark_mode_override(_exe_dir: &Path, dark: Option<bool>) {
    let exe_dir = get_exe_dir();
    let host = read_config_listen_host(&exe_dir);
    let use_tray = read_config_use_tray(&exe_dir);
    let config_path = exe_dir.join("config.toml");
    let dark_str = match dark {
        Some(true) => "dark",
        Some(false) => "light",
        None => "system",
    };
    let content = format!(
        "listen_addr = \"{}:10882\"\nmcp_listen_addr = \"{}:10881\"\ndb_path = \"\"\n\n[mcp]\ntools_path = \"tools\"\nskills_dir = \"skills\"\n\n[log]\nlevel = \"info\"\n\n[gui]\nuse_tray = {}\ndark_mode = \"{}\"\n",
        host, host, use_tray, dark_str
    );
    std::fs::write(config_path, content).ok();
}

pub fn read_auto_start() -> bool {
    #[cfg(windows)]
    {
        use winreg::enums::*;
        use winreg::RegKey;
        let hkcu = RegKey::predef(HKEY_CURRENT_USER);
        hkcu.open_subkey("Software\\Microsoft\\Windows\\CurrentVersion\\Run")
            .and_then(|key| key.get_value::<String, _>("AISkillBox"))
            .is_ok()
    }
    #[cfg(not(windows))]
    {
        false
    }
}

pub fn toggle_auto_start(enable: bool) {
    #[cfg(windows)]
    {
        use winreg::enums::*;
        use winreg::RegKey;
        let hkcu = RegKey::predef(HKEY_CURRENT_USER);
        let key = hkcu
            .open_subkey_with_flags("Software\\Microsoft\\Windows\\CurrentVersion\\Run", KEY_ALL_ACCESS)
            .unwrap();
        if enable {
            let path = std::env::current_exe()
                .unwrap()
                .to_string_lossy()
                .to_string();
            key.set_value("AISkillBox", &path).ok();
        } else {
            key.delete_value("AISkillBox").ok();
        }
    }
}

pub fn copy_skill_dir(src: &Path, dst: &Path) -> Result<(), String> {
    if dst.exists() {
        return Err(format!("{} 已存在", dst.file_name().unwrap_or_default().to_string_lossy()));
    }
    copy_dir_all(src, dst).map_err(|e| format!("复制失败: {}", e))
}

fn get_exe_dir() -> PathBuf {
    std::env::current_exe()
        .ok()
        .and_then(|e| e.parent().map(|p| p.to_path_buf()))
        .unwrap_or_default()
}

pub fn copy_dir_all(src: &Path, dst: &Path) -> std::io::Result<()> {
    std::fs::create_dir_all(dst)?;
    for entry in std::fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        if ty.is_dir() {
            copy_dir_all(&entry.path(), &dst.join(entry.file_name()))?;
        } else {
            std::fs::copy(entry.path(), dst.join(entry.file_name()))?;
        }
    }
    Ok(())
}
