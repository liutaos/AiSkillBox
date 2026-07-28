// Copyright (c) Mr_老鬼. All rights reserved.
// https://www.junjiestudio.top

pub mod file_handler;

use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

use crate::tools::schema::AppError;

pub type ToolResult = Result<serde_json::Value, AppError>;

pub type ToolHandler = Arc<
    dyn Fn(serde_json::Value) -> Pin<Box<dyn Future<Output = ToolResult> + Send>>
        + Send
        + Sync,
>;

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

    pub fn from_handlers(handlers: HashMap<String, ToolHandler>) -> Self {
        Self {
            handlers: Arc::new(handlers),
        }
    }

    pub async fn execute(&self, name: &str, args: serde_json::Value) -> ToolResult {
        if let Some(handler) = self.handlers.get(name) {
            handler(args).await
        } else {
            Err(AppError::ToolNotFound(name.to_string()))
        }
    }
}

impl Default for ToolExecutor {
    fn default() -> Self {
        Self::new()
    }
}
