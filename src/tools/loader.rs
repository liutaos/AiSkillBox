use std::path::{Path, PathBuf};
use tracing::{info, warn};

use super::schema::{ToolConfig, AppError};
use super::skill_scanner::{self, SkillInfo};

/// 配置加载器（支持 JSON 配置 + Skills 自动扫描）
#[derive(Clone)]
pub struct JsonLoader {
    tools_path: PathBuf,
}

impl JsonLoader {
    pub fn new(tools_path: &str) -> Self {
        Self {
            tools_path: PathBuf::from(tools_path),
        }
    }

    /// 加载所有 JSON 配置
    pub fn load_tools(&self) -> Result<Vec<ToolConfig>, AppError> {
        let mut configs = Vec::new();

        if !self.tools_path.exists() {
            warn!("工具配置目录不存在: {:?}", self.tools_path);
            return Ok(configs);
        }

        let entries = std::fs::read_dir(&self.tools_path)?;

        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() && path.extension().map(|e| e == "json").unwrap_or(false) {
                match ToolConfig::from_file(&path) {
                    Ok(config) => {
                        info!("加载工具配置: {:?} ({} 个工具)", path.file_name(), config.tools.len());
                        configs.push(config);
                    }
                    Err(e) => {
                        warn!("加载配置失败: {:?} - {}", path, e);
                    }
                }
            }
        }

        Ok(configs)
    }

    /// 扫描 skills 目录，自动生成工具配置
    pub fn load_skills(&self, skills_dir: &str) -> Vec<SkillInfo> {
        skill_scanner::scan_skills(skills_dir)
    }

    /// 兼容旧接口：加载所有（JSON 配置）
    pub fn load_all(&self) -> Result<Vec<ToolConfig>, AppError> {
        self.load_tools()
    }

    /// 获取工具配置目录
    pub fn tools_path(&self) -> &Path {
        &self.tools_path
    }
}
