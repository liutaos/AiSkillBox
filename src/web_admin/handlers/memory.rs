// Copyright (c) Mr_老鬼. All rights reserved.
// https://www.junjiestudio.top
// Derivative works must retain this copyright notice.

use salvo::prelude::*;
use serde::Deserialize;
use std::sync::Arc;

use crate::db::{SkillDb, MemoryStore, IssueStore};

/// 搜索记忆请求
#[derive(Deserialize)]
pub struct SearchMemoryRequest {
    pub query: String,
    pub tags: Option<Vec<String>>,
    pub include_archived: Option<bool>,
    pub limit: Option<i64>,
}

/// 删除记忆请求
#[derive(Deserialize)]
pub struct DeleteMemoryRequest {
    pub memory_id: i64,
}

/// 归档记忆请求
#[derive(Deserialize)]
pub struct ArchiveMemoryRequest {
    pub older_than_days: Option<i64>,
}

/// 搜索问题请求
#[derive(Deserialize)]
pub struct SearchIssueRequest {
    pub query: String,
    pub task: Option<String>,
    pub category: Option<String>,
    pub limit: Option<i64>,
}

/// 删除问题请求
#[derive(Deserialize)]
pub struct DeleteIssueRequest {
    pub issue_id: i64,
}

/// 分页参数
fn parse_pagination(req: &Request) -> (usize, usize) {
    let page = req.query::<i64>("page")
        .and_then(|p| if p > 0 { Some(p as usize) } else { None })
        .unwrap_or(1);
    let size = req.query::<i64>("size")
        .and_then(|s| if s > 0 { Some(s as usize) } else { None })
        .unwrap_or(20);
    (page, size)
}

/// 对记忆列表进行内存分页
fn paginate_memories(
    memories: Vec<crate::db::Memory>,
    page: usize,
    size: usize,
) -> (Vec<crate::db::Memory>, usize) {
    let total = memories.len();
    let start = (page - 1) * size;
    if start >= total { return (Vec::new(), total); }
    let end = (start + size).min(total);
    (memories[start..end].to_vec(), total)
}

/// 对问题列表进行内存分页
fn paginate_issues(
    issues: Vec<crate::db::Issue>,
    page: usize,
    size: usize,
) -> (Vec<crate::db::Issue>, usize) {
    let total = issues.len();
    let start = (page - 1) * size;
    if start >= total { return (Vec::new(), total); }
    let end = (start + size).min(total);
    (issues[start..end].to_vec(), total)
}

// ============ 记忆 API ============

/// 列出记忆
#[handler]
pub async fn list_memories(req: &mut Request, depot: &mut Depot, res: &mut Response) {
    let db = depot.get_typed::<Arc<SkillDb>>().unwrap();
    let (page, size) = parse_pagination(req);

    let all = db.recall("", None, false, 10000).unwrap_or_default();
    let (paged, total) = paginate_memories(all, page, size);
    res.render(Json(serde_json::json!({
        "success": true,
        "data": { "count": total, "memories": paged }
    })));
}

/// 搜索记忆
#[handler]
pub async fn search_memories(req: &mut Request, depot: &mut Depot, res: &mut Response) {
    let db = depot.get_typed::<Arc<SkillDb>>().unwrap();

    let body: SearchMemoryRequest = match req.parse_json().await {
        Ok(b) => b,
        Err(e) => {
            res.status_code(StatusCode::BAD_REQUEST);
            res.render(Json(serde_json::json!({
                "success": false, "message": format!("请求解析失败: {}", e)
            })));
            return;
        }
    };

    let include_archived = body.include_archived.unwrap_or(false);
    let limit = body.limit.unwrap_or(50);

    match db.recall(&body.query, body.tags, include_archived, limit) {
        Ok(memories) => {
            res.render(Json(serde_json::json!({
                "success": true,
                "data": { "count": memories.len(), "memories": memories }
            })));
        }
        Err(e) => {
            res.status_code(StatusCode::INTERNAL_SERVER_ERROR);
            res.render(Json(serde_json::json!({
                "success": false, "message": format!("搜索失败: {}", e)
            })));
        }
    }
}

/// 删除记忆
#[handler]
pub async fn delete_memory(req: &mut Request, depot: &mut Depot, res: &mut Response) {
    let db = depot.get_typed::<Arc<SkillDb>>().unwrap();

    let body: DeleteMemoryRequest = match req.parse_json().await {
        Ok(b) => b,
        Err(e) => {
            res.status_code(StatusCode::BAD_REQUEST);
            res.render(Json(serde_json::json!({
                "success": false, "message": format!("请求解析失败: {}", e)
            })));
            return;
        }
    };

    match db.forget(body.memory_id) {
        Ok(()) => {
            res.render(Json(serde_json::json!({
                "success": true, "message": format!("已删除记忆 ID: {}", body.memory_id)
            })));
        }
        Err(e) => {
            res.status_code(StatusCode::INTERNAL_SERVER_ERROR);
            res.render(Json(serde_json::json!({
                "success": false, "message": format!("删除失败: {}", e)
            })));
        }
    }
}

/// 归档记忆
#[handler]
pub async fn archive_memories(req: &mut Request, depot: &mut Depot, res: &mut Response) {
    let db = depot.get_typed::<Arc<SkillDb>>().unwrap();

    let body: ArchiveMemoryRequest = match req.parse_json().await {
        Ok(b) => b,
        Err(_) => ArchiveMemoryRequest { older_than_days: Some(30) },
    };

    let days = body.older_than_days.unwrap_or(30);
    match db.archive(days) {
        Ok(count) => {
            res.render(Json(serde_json::json!({
                "success": true,
                "data": { "archived_count": count, "message": format!("已归档 {} 条记忆", count) }
            })));
        }
        Err(e) => {
            res.status_code(StatusCode::INTERNAL_SERVER_ERROR);
            res.render(Json(serde_json::json!({
                "success": false, "message": format!("归档失败: {}", e)
            })));
        }
    }
}

/// 记忆统计
#[handler]
pub async fn get_memory_stats(_req: &mut Request, depot: &mut Depot, res: &mut Response) {
    let db = depot.get_typed::<Arc<SkillDb>>().unwrap();

    match db.get_stats() {
        Ok(stats) => {
            res.render(Json(serde_json::json!({
                "success": true,
                "data": {
                    "total": stats.total,
                    "active": stats.active,
                    "archived": stats.archived,
                    "archive_rate": format!("{:.2}%", stats.archive_rate * 100.0)
                }
            })));
        }
        Err(e) => {
            res.status_code(StatusCode::INTERNAL_SERVER_ERROR);
            res.render(Json(serde_json::json!({
                "success": false, "message": format!("获取统计失败: {}", e)
            })));
        }
    }
}

// ============ 问题 API ============

/// 列出问题
#[handler]
pub async fn list_issues(req: &mut Request, depot: &mut Depot, res: &mut Response) {
    let db = depot.get_typed::<Arc<SkillDb>>().unwrap();
    let (page, size) = parse_pagination(req);

    let all = db.search_issues("", None, None, 10000).unwrap_or_default();
    let (paged, total) = paginate_issues(all, page, size);
    res.render(Json(serde_json::json!({
        "success": true,
        "data": { "count": total, "issues": paged }
    })));
}

/// 搜索问题
#[handler]
pub async fn search_issues(req: &mut Request, depot: &mut Depot, res: &mut Response) {
    let db = depot.get_typed::<Arc<SkillDb>>().unwrap();

    let body: SearchIssueRequest = match req.parse_json().await {
        Ok(b) => b,
        Err(e) => {
            res.status_code(StatusCode::BAD_REQUEST);
            res.render(Json(serde_json::json!({
                "success": false, "message": format!("请求解析失败: {}", e)
            })));
            return;
        }
    };

    let limit = body.limit.unwrap_or(50);
    match db.search_issues(&body.query, body.task.as_deref(), body.category.as_deref(), limit) {
        Ok(issues) => {
            res.render(Json(serde_json::json!({
                "success": true,
                "data": { "count": issues.len(), "issues": issues }
            })));
        }
        Err(e) => {
            res.status_code(StatusCode::INTERNAL_SERVER_ERROR);
            res.render(Json(serde_json::json!({
                "success": false, "message": format!("搜索失败: {}", e)
            })));
        }
    }
}

/// 删除问题
#[handler]
pub async fn delete_issue(req: &mut Request, depot: &mut Depot, res: &mut Response) {
    let db = depot.get_typed::<Arc<SkillDb>>().unwrap();

    let body: DeleteIssueRequest = match req.parse_json().await {
        Ok(b) => b,
        Err(e) => {
            res.status_code(StatusCode::BAD_REQUEST);
            res.render(Json(serde_json::json!({
                "success": false, "message": format!("请求解析失败: {}", e)
            })));
            return;
        }
    };

    let conn = db.get_conn();
    match conn.execute("DELETE FROM common_issues WHERE id = ?1", [body.issue_id]) {
        Ok(_) => {
            res.render(Json(serde_json::json!({
                "success": true, "message": format!("已删除问题 ID: {}", body.issue_id)
            })));
        }
        Err(e) => {
            res.status_code(StatusCode::INTERNAL_SERVER_ERROR);
            res.render(Json(serde_json::json!({
                "success": false, "message": format!("删除失败: {}", e)
            })));
        }
    }
}
