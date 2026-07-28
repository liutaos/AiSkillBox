use serde::{Deserialize, Serialize};
use std::sync::OnceLock;

static BASE_URL: OnceLock<String> = OnceLock::new();

pub fn init_base_url(addr: &str) {
    let _ = BASE_URL.set(format!("http://{}/api/admin", addr));
}

fn base_url() -> String {
    BASE_URL.get().cloned().unwrap_or_else(|| "http://127.0.0.1:10882/api/admin".to_string())
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillEntry {
    pub name: String,
    pub description: String,
    #[serde(default)]
    pub tags: String,
    pub dir_name: String,
    pub enabled: bool,
}

#[derive(Deserialize)]
struct ApiResponse<T> {
    success: bool,
    data: Option<T>,
    message: Option<String>,
}

#[derive(Deserialize)]
struct SkillsData {
    count: usize,
    skills: Vec<SkillEntry>,
}

#[derive(Deserialize)]
struct StatusData {
    running: bool,
}

pub fn list_skills() -> Result<Vec<SkillEntry>, String> {
    let url = format!("{}/skills", base_url());
    let resp = reqwest::blocking::get(&url)
        .map_err(|e| format!("请求失败: {}", e))?;
    let json: ApiResponse<SkillsData> = resp.json()
        .map_err(|e| format!("解析失败: {}", e))?;
    if json.success {
        Ok(json.data.map(|d| d.skills).unwrap_or_default())
    } else {
        Err(json.message.unwrap_or("未知错误".to_string()))
    }
}

pub fn delete_skill(name: &str) -> Result<(), String> {
    let url = format!("{}/delete", base_url());
    let body = serde_json::json!({ "skill_name": name });
    let resp = reqwest::blocking::Client::new()
        .post(&url)
        .json(&body)
        .send()
        .map_err(|e| format!("请求失败: {}", e))?;
    let json: ApiResponse<()> = resp.json()
        .map_err(|e| format!("解析失败: {}", e))?;
    if json.success {
        Ok(())
    } else {
        Err(json.message.unwrap_or("删除失败".to_string()))
    }
}

pub fn enable_skill(name: &str) -> Result<(), String> {
    let url = format!("{}/enable", base_url());
    let body = serde_json::json!({ "skill_name": name });
    let resp = reqwest::blocking::Client::new()
        .post(&url)
        .json(&body)
        .send()
        .map_err(|e| format!("请求失败: {}", e))?;
    let json: ApiResponse<()> = resp.json()
        .map_err(|e| format!("解析失败: {}", e))?;
    if json.success {
        Ok(())
    } else {
        Err(json.message.unwrap_or("启用失败".to_string()))
    }
}

pub fn disable_skill(name: &str) -> Result<(), String> {
    let url = format!("{}/disable", base_url());
    let body = serde_json::json!({ "skill_name": name });
    let resp = reqwest::blocking::Client::new()
        .post(&url)
        .json(&body)
        .send()
        .map_err(|e| format!("请求失败: {}", e))?;
    let json: ApiResponse<()> = resp.json()
        .map_err(|e| format!("解析失败: {}", e))?;
    if json.success {
        Ok(())
    } else {
        Err(json.message.unwrap_or("禁用失败".to_string()))
    }
}

pub fn check_status() -> bool {
    let url = format!("{}/status", base_url());
    let resp = match reqwest::blocking::get(&url) {
        Ok(r) => r,
        Err(_) => return false,
    };
    let json: ApiResponse<StatusData> = match resp.json() {
        Ok(j) => j,
        Err(_) => return false,
    };
    json.data.map(|d| d.running).unwrap_or(false)
}

pub fn start_service() -> Result<(), String> {
    let url = format!("{}/start", base_url());
    let resp = reqwest::blocking::Client::new()
        .post(&url)
        .send()
        .map_err(|e| format!("请求失败: {}", e))?;
    let json: ApiResponse<()> = resp.json()
        .map_err(|e| format!("解析失败: {}", e))?;
    if json.success {
        Ok(())
    } else {
        Err(json.message.unwrap_or("启动失败".to_string()))
    }
}

pub fn stop_service() -> Result<(), String> {
    let url = format!("{}/stop", base_url());
    let resp = reqwest::blocking::Client::new()
        .post(&url)
        .send()
        .map_err(|e| format!("请求失败: {}", e))?;
    let json: ApiResponse<()> = resp.json()
        .map_err(|e| format!("解析失败: {}", e))?;
    if json.success {
        Ok(())
    } else {
        Err(json.message.unwrap_or("停止失败".to_string()))
    }
}

pub fn refresh_skills() -> Result<(), String> {
    let url = format!("{}/refresh", base_url());
    let resp = reqwest::blocking::Client::new()
        .post(&url)
        .send()
        .map_err(|e| format!("请求失败: {}", e))?;
    let json: ApiResponse<()> = resp.json()
        .map_err(|e| format!("解析失败: {}", e))?;
    if json.success {
        Ok(())
    } else {
        Err(json.message.unwrap_or("刷新失败".to_string()))
    }
}
