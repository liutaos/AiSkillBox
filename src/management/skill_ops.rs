// Copyright (c) Mr_老鬼. All rights reserved.
// https://www.junjiestudio.top
// Derivative works must retain this copyright notice.

use std::sync::Arc;
use tracing::info;
use crate::db::{SkillDb, SkillStore};

pub fn delete_skill(db: &Arc<SkillDb>, skills_dir: &str, skill_name: &str) -> Result<String, String> {
    let skills_path = std::path::Path::new(skills_dir);
    let skill_dir = skills_path.join(skill_name);
    
    if !skill_dir.exists() {
        return Err(format!("Skill '{}' 不存在", skill_name));
    }
    
    let trash_dir = skills_path.parent()
        .unwrap_or(skills_path)
        .join("skill-trash");
    std::fs::create_dir_all(&trash_dir)
        .map_err(|e| format!("创建回收站失败: {}", e))?;
    
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    let trash_path = trash_dir.join(format!("{}_{}", skill_name, timestamp));
    
    std::fs::rename(&skill_dir, &trash_path)
        .map_err(|e| format!("移动到回收站失败: {}", e))?;
    
    info!("Skill '{}' 已移动到回收站: {:?}", skill_name, trash_path);
    
    db.soft_delete(skill_name)
        .map_err(|e| format!("更新数据库失败: {}", e))?;
    
    Ok(format!("Skill '{}' 已删除，移至回收站", skill_name))
}

pub fn restore_skill(db: &Arc<SkillDb>, skills_dir: &str, skill_name: &str) -> Result<String, String> {
    let skills_path = std::path::Path::new(skills_dir);
    let trash_dir = skills_path.parent()
        .unwrap_or(skills_path)
        .join("skill-trash");
    
    let mut found_dir = None;
    if trash_dir.exists() {
        for entry in std::fs::read_dir(&trash_dir).map_err(|e| format!("读取回收站失败: {}", e))? {
            let entry = entry.map_err(|e| format!("读取回收站条目失败: {}", e))?;
            let path = entry.path();
            if path.is_dir() {
                let dir_name = path.file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("");
                if dir_name == skill_name || dir_name.starts_with(&format!("{}_", skill_name)) {
                    found_dir = Some(path);
                    break;
                }
            }
        }
    }
    
    let source_dir = found_dir
        .ok_or_else(|| format!("Skill '{}' 在回收站中不存在", skill_name))?;
    
    let target_dir = skills_path.join(skill_name);
    if target_dir.exists() {
        return Err(format!("目标目录已存在: {:?}", target_dir));
    }
    
    std::fs::rename(&source_dir, &target_dir)
        .map_err(|e| format!("恢复失败: {}", e))?;
    
    info!("Skill '{}' 已从回收站恢复", skill_name);
    
    db.restore(skill_name)
        .map_err(|e| format!("更新数据库失败: {}", e))?;
    
    Ok(format!("Skill '{}' 已恢复", skill_name))
}

pub fn permanent_delete_skill(db: &Arc<SkillDb>, skills_dir: &str, skill_name: &str) -> Result<String, String> {
    let skills_path = std::path::Path::new(skills_dir);
    let trash_dir = skills_path.parent()
        .unwrap_or(skills_path)
        .join("skill-trash");
    
    let mut found_dir = None;
    if trash_dir.exists() {
        for entry in std::fs::read_dir(&trash_dir).map_err(|e| format!("读取回收站失败: {}", e))? {
            let entry = entry.map_err(|e| format!("读取回收站条目失败: {}", e))?;
            let path = entry.path();
            if path.is_dir() {
                let dir_name = path.file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("");
                if dir_name == skill_name || dir_name.starts_with(&format!("{}_", skill_name)) {
                    found_dir = Some(path);
                    break;
                }
            }
        }
    }
    
    if let Some(dir) = found_dir {
        std::fs::remove_dir_all(&dir)
            .map_err(|e| format!("删除目录失败: {}", e))?;
        info!("Skill '{}' 目录已永久删除: {:?}", skill_name, dir);
    }
    
    db.permanent_delete(skill_name)
        .map_err(|e| format!("删除数据库记录失败: {}", e))?;
    
    Ok(format!("Skill '{}' 已永久删除", skill_name))
}

pub fn enable_skill(db: &Arc<SkillDb>, skill_name: &str) -> Result<String, String> {
    db.set_enabled(skill_name, true)
        .map_err(|e| format!("启用失败: {}", e))?;
    info!("Skill '{}' 已启用", skill_name);
    Ok(format!("Skill '{}' 已启用", skill_name))
}

pub fn disable_skill(db: &Arc<SkillDb>, skill_name: &str) -> Result<String, String> {
    db.set_enabled(skill_name, false)
        .map_err(|e| format!("禁用失败: {}", e))?;
    info!("Skill '{}' 已禁用", skill_name);
    Ok(format!("Skill '{}' 已禁用", skill_name))
}
