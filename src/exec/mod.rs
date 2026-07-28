pub mod file_handler;

use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

use crate::tools::schema::AppError;

/// 工具执行结果
pub type ToolResult = Result<serde_json::Value, AppError>;

/// 工具执行函数类型
pub type ToolHandler = Arc<
    dyn Fn(serde_json::Value) -> Pin<Box<dyn Future<Output = ToolResult> + Send>>
        + Send
        + Sync,
>;

/// 工具执行器 - 统一管理工具执行逻辑
#[derive(Clone)]
pub struct ToolExecutor {
    handlers: Arc<HashMap<String, ToolHandler>>,
}

impl ToolExecutor {
    pub fn new() -> Self {
        Self {
            handlers: Arc::new(HashMap::new()),
        }
    }

    /// 从 handler 映射创建执行器
    pub fn from_handlers(handlers: HashMap<String, ToolHandler>) -> Self {
        Self {
            handlers: Arc::new(handlers),
        }
    }

    /// 执行工具
    pub async fn execute(&self, name: &str, args: serde_json::Value) -> ToolResult {
        if let Some(handler) = self.handlers.get(name) {
            handler(args).await
        } else {
            Err(AppError::ToolNotFound(name.to_string()))
        }
    }

    /// 检查工具是否存在
    pub fn has_tool(&self, name: &str) -> bool {
        self.handlers.contains_key(name)
    }

    /// 获取所有已注册的工具名称
    pub fn tool_names(&self) -> Vec<String> {
        self.handlers.keys().cloned().collect()
    }
}

impl Default for ToolExecutor {
    fn default() -> Self {
        Self::new()
    }
}
