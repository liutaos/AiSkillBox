use std::sync::OnceLock;

use figment::Figment;
use figment::providers::{Env, Format, Toml};
use serde::Deserialize;

pub static CONFIG: OnceLock<ServerConfig> = OnceLock::new();

pub fn init() {
    // 获取 exe 所在目录
    let exe_dir = std::env::current_exe()
        .ok()
        .and_then(|exe| exe.parent().map(|p| p.to_path_buf()))
        .unwrap_or_else(|| std::env::current_dir().unwrap_or_default());

    // 查找 config.toml
    let config_path = {
        let exe_config = exe_dir.join("config.toml");
        if exe_config.exists() {
            exe_config.to_string_lossy().to_string()
        } else {
            "config.toml".to_string()
        }
    };

    let raw_config = Figment::new()
        .merge(Toml::file(
            Env::var("APP_CONFIG").as_deref().unwrap_or(&config_path),
        ))
        .merge(Env::prefixed("APP_").global());

    let mut config = match raw_config.extract::<ServerConfig>() {
        Ok(s) => s,
        Err(e) => {
            eprintln!("配置解析错误: {e}");
            eprintln!("配置路径: {}", config_path);
            std::process::exit(1);
        }
    };

    // 解析相对路径为绝对路径
    if !std::path::Path::new(&config.mcp.tools_path).is_absolute() {
        config.mcp.tools_path = exe_dir.join(&config.mcp.tools_path).to_string_lossy().to_string();
    }

    if !std::path::Path::new(&config.mcp.skills_dir).is_absolute() {
        config.mcp.skills_dir = exe_dir.join(&config.mcp.skills_dir).to_string_lossy().to_string();
    }

    if config.db_path.is_empty() {
        config.db_path = exe_dir.join("skills.db").to_string_lossy().to_string();
    }

    crate::config::CONFIG
        .set(config)
        .expect("config should be set");
}

pub fn get() -> &'static ServerConfig {
    CONFIG.get().expect("config should be set")
}

#[derive(Deserialize, Clone, Debug)]
pub struct ServerConfig {
    #[serde(default = "default_listen_addr")]
    pub listen_addr: String,

    #[serde(default = "default_mcp_listen_addr")]
    pub mcp_listen_addr: String,

    #[serde(default)]
    pub db_path: String,

    #[serde(default)]
    pub mcp: McpConfig,

    #[serde(default)]
    pub log: LogConfig,
}

#[derive(Deserialize, Clone, Debug)]
pub struct McpConfig {
    #[serde(default = "default_tools_path")]
    pub tools_path: String,

    #[serde(default = "default_skills_dir")]
    pub skills_dir: String,
}

#[derive(Deserialize, Clone, Debug)]
pub struct LogConfig {
    #[serde(default = "default_log_level")]
    pub level: String,
}

impl Default for McpConfig {
    fn default() -> Self {
        Self {
            tools_path: default_tools_path(),
            skills_dir: default_skills_dir(),
        }
    }
}

impl Default for LogConfig {
    fn default() -> Self {
        Self {
            level: default_log_level(),
        }
    }
}

fn default_listen_addr() -> String {
    "127.0.0.1:10882".into()
}

fn default_mcp_listen_addr() -> String {
    "127.0.0.1:10881".into()
}

fn default_tools_path() -> String {
    "tools".into()
}

fn default_skills_dir() -> String {
    "skills".into()
}

fn default_log_level() -> String {
    "info".into()
}
