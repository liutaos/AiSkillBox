use serde::Deserialize;
use std::fmt;

/// 统一错误类型
#[derive(Debug)]
pub enum AppError {
    Io(std::io::Error),
    Json(serde_json::Error),
    Config(String),
    ToolNotFound(String),
    ToolExecFailed(String),
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AppError::Io(e) => write!(f, "IO错误: {}", e),
            AppError::Json(e) => write!(f, "JSON解析错误: {}", e),
            AppError::Config(msg) => write!(f, "配置错误: {}", msg),
            AppError::ToolNotFound(name) => write!(f, "工具不存在: {}", name),
            AppError::ToolExecFailed(msg) => write!(f, "工具执行失败: {}", msg),
        }
    }
}

impl std::error::Error for AppError {}

impl From<std::io::Error> for AppError {
    fn from(e: std::io::Error) -> Self {
        AppError::Io(e)
    }
}

impl From<serde_json::Error> for AppError {
    fn from(e: serde_json::Error) -> Self {
        AppError::Json(e)
    }
}

/// 工具配置文件格式
#[derive(Deserialize, Clone, Debug)]
pub struct ToolConfig {
    #[serde(default)]
    pub version: String,
    #[serde(default)]
    pub tools: Vec<ToolDefinition>,
}

/// 工具定义
#[derive(Deserialize, Clone, Debug)]
pub struct ToolDefinition {
    pub name: String,
    pub description: String,
    #[serde(default, rename = "inputSchema")]
    pub input_schema: serde_json::Value,
    #[serde(default)]
    pub handler: String,
}

impl ToolConfig {
    /// 从 JSON 字符串解析
    pub fn from_json(json: &str) -> Result<Self, AppError> {
        Ok(serde_json::from_str(json)?)
    }

    /// 从文件加载
    pub fn from_file(path: &std::path::Path) -> Result<Self, AppError> {
        let content = std::fs::read_to_string(path)?;
        let config = Self::from_json(&content)?;
        Ok(config)
    }
}
