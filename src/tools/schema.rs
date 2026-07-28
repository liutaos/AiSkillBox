use serde::Deserialize;
use std::fmt;

#[derive(Debug)]
pub enum AppError {
    Io(std::io::Error),
    Json(serde_json::Error),
    ToolNotFound(String),
    ToolExecFailed(String),
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AppError::Io(e) => write!(f, "IO错误: {}", e),
            AppError::Json(e) => write!(f, "JSON解析错误: {}", e),
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

#[derive(Deserialize, Clone, Debug)]
pub struct ToolConfig {
    #[serde(default)]
    #[allow(dead_code)]
    pub version: String,
    #[serde(default)]
    pub tools: Vec<ToolDefinition>,
}

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
    pub fn from_json(json: &str) -> Result<Self, AppError> {
        Ok(serde_json::from_str(json)?)
    }

    pub fn from_file(path: &std::path::Path) -> Result<Self, AppError> {
        let content = std::fs::read_to_string(path)?;
        let config = Self::from_json(&content)?;
        Ok(config)
    }
}
