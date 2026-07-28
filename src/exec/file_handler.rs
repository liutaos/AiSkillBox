// Copyright (c) Mr_老鬼. All rights reserved.
// https://www.junjiestudio.top

use std::path::PathBuf;
use std::sync::Arc;

use super::ToolHandler;
use crate::tools::schema::AppError;

pub struct FileHandlerFactory {
    base_path: PathBuf,
}

impl FileHandlerFactory {
    pub fn new(base_path: &str) -> Self {
        Self {
            base_path: PathBuf::from(base_path),
        }
    }

    pub fn create_file_handler(&self, file_path: &str) -> ToolHandler {
        let base_path = self.base_path.clone();
        let file_path = file_path.to_string();

        make_handler(move |_args| {
            let base_path = base_path.clone();
            let file_path = file_path.clone();
            async move {
                let full_path = base_path.join(&file_path);

                if !full_path.exists() {
                    return Err(AppError::ToolExecFailed(
                        format!("文件不存在: {}", full_path.display())
                    ));
                }

                let content = std::fs::read_to_string(&full_path)
                    .map_err(|e| AppError::ToolExecFailed(
                        format!("读取文件失败: {} - {}", full_path.display(), e)
                    ))?;

                Ok(serde_json::json!({
                    "file": file_path,
                    "content": content
                }))
            }
        })
    }
}

fn make_handler<F, Fut>(f: F) -> ToolHandler
where
    F: Fn(serde_json::Value) -> Fut + Send + Sync + 'static,
    Fut: std::future::Future<Output = Result<serde_json::Value, AppError>> + Send + 'static,
{
    Arc::new(move |args| Box::pin(f(args)))
}
