// Copyright (c) Mr_老鬼. All rights reserved.
// https://www.junjiestudio.top

use salvo::prelude::*;
use serde::Deserialize;
use std::sync::Arc;

use crate::db::{SkillDb, SkillStore};
use crate::management::skill_ops;

/// 搜索请求
#[derive(Deserialize)]
pub struct SearchRequest {
    pub query: String,
    pub tags: Option<String>,
}

/// 启用/禁用请求
#[derive(Deserialize)]
pub struct EnableRequest {
    pub skill_name: String,
}

/// 删除请求
#[derive(Deserialize)]
pub struct DeleteRequest {
    pub skill_name: String,
}

/// 解析分页参数
fn parse_pagination(req: &Request) -> (usize, usize) {
    let page = req
        .query::<i64>("page")
        .and_then(|p| if p > 0 { Some(p as usize) } else { None })
        .unwrap_or(1);
    let size = req
        .query::<i64>("size")
        .and_then(|s| if s > 0 { Some(s as usize) } else { None })
        .unwrap_or(20);
    (page, size)
}

/// 对 skill 列表进行内存分页
fn paginate_skills(
    skills: Vec<crate::db::Skill>,
    page: usize,
    size: usize,
) -> (Vec<crate::db::Skill>, usize) {
    let total = skills.len();
    let start = (page - 1) * size;
    if start >= total {
        return (Vec::new(), total);
    }
    let end = (start + size).min(total);
    (skills[start..end].to_vec(), total)
}

/// 列出所有 skill
#[handler]
pub async fn list_skills(req: &mut Request, depot: &mut Depot, res: &mut Response) {
    let db = depot.get_typed::<Arc<SkillDb>>().unwrap();
    let (page, size) = parse_pagination(req);

    match db.list_all() {
        Ok(skills) => {
            let (paged_skills, total) = paginate_skills(skills, page, size);
            res.render(Json(serde_json::json!({
                "success": true,
                "data": {
                    "count": total,
                    "skills": paged_skills
                }
            })));
        }
        Err(e) => {
            res.status_code(StatusCode::INTERNAL_SERVER_ERROR);
            res.render(Json(serde_json::json!({
                "success": false,
                "message": format!("查询失败: {}", e)
            })));
        }
    }
}

/// 列出回收站
#[handler]
pub async fn list_trash(req: &mut Request, depot: &mut Depot, res: &mut Response) {
    let db = depot.get_typed::<Arc<SkillDb>>().unwrap();
    let (page, size) = parse_pagination(req);

    match db.list_trash() {
        Ok(skills) => {
            let (paged_skills, total) = paginate_skills(skills, page, size);
            res.render(Json(serde_json::json!({
                "success": true,
                "data": {
                    "count": total,
                    "skills": paged_skills
                }
            })));
        }
        Err(e) => {
            res.status_code(StatusCode::INTERNAL_SERVER_ERROR);
            res.render(Json(serde_json::json!({
                "success": false,
                "message": format!("查询失败: {}", e)
            })));
        }
    }
}

/// 搜索 skill
#[handler]
pub async fn search_skills(req: &mut Request, depot: &mut Depot, res: &mut Response) {
    let db = depot.get_typed::<Arc<SkillDb>>().unwrap();

    let body: SearchRequest = match req.parse_json().await {
        Ok(b) => b,
        Err(e) => {
            res.status_code(StatusCode::BAD_REQUEST);
            res.render(Json(serde_json::json!({
                "success": false,
                "message": format!("请求解析失败: {}", e)
            })));
            return;
        }
    };

    match db.search(&body.query, body.tags.as_deref()) {
        Ok(skills) => {
            res.render(Json(serde_json::json!({
                "success": true,
                "data": {
                    "count": skills.len(),
                    "skills": skills
                }
            })));
        }
        Err(e) => {
            res.status_code(StatusCode::INTERNAL_SERVER_ERROR);
            res.render(Json(serde_json::json!({
                "success": false,
                "message": format!("搜索失败: {}", e)
            })));
        }
    }
}

/// 删除 skill 到回收站
#[handler]
pub async fn delete_skill(req: &mut Request, depot: &mut Depot, res: &mut Response) {
    let db = depot.get_typed::<Arc<SkillDb>>().unwrap();
    let skills_dir = depot.get_typed::<String>().unwrap().clone();

    let body: DeleteRequest = match req.parse_json().await {
        Ok(b) => b,
        Err(e) => {
            res.status_code(StatusCode::BAD_REQUEST);
            res.render(Json(serde_json::json!({
                "success": false,
                "message": format!("请求解析失败: {}", e)
            })));
            return;
        }
    };

    match skill_ops::delete_skill(&db, &skills_dir, &body.skill_name) {
        Ok(msg) => {
            res.render(Json(serde_json::json!({
                "success": true,
                "message": msg
            })));
        }
        Err(e) => {
            res.status_code(StatusCode::INTERNAL_SERVER_ERROR);
            res.render(Json(serde_json::json!({
                "success": false,
                "message": e
            })));
        }
    }
}

/// 从回收站恢复 skill
#[handler]
pub async fn restore_skill(req: &mut Request, depot: &mut Depot, res: &mut Response) {
    let db = depot.get_typed::<Arc<SkillDb>>().unwrap();
    let skills_dir = depot.get_typed::<String>().unwrap().clone();

    let body: DeleteRequest = match req.parse_json().await {
        Ok(b) => b,
        Err(e) => {
            res.status_code(StatusCode::BAD_REQUEST);
            res.render(Json(serde_json::json!({
                "success": false,
                "message": format!("请求解析失败: {}", e)
            })));
            return;
        }
    };

    match skill_ops::restore_skill(&db, &skills_dir, &body.skill_name) {
        Ok(msg) => {
            res.render(Json(serde_json::json!({
                "success": true,
                "message": msg
            })));
        }
        Err(e) => {
            res.status_code(StatusCode::INTERNAL_SERVER_ERROR);
            res.render(Json(serde_json::json!({
                "success": false,
                "message": e
            })));
        }
    }
}

/// 永久删除 skill
#[handler]
pub async fn permanent_delete_skill(req: &mut Request, depot: &mut Depot, res: &mut Response) {
    let db = depot.get_typed::<Arc<SkillDb>>().unwrap();
    let skills_dir = depot.get_typed::<String>().unwrap().clone();

    let body: DeleteRequest = match req.parse_json().await {
        Ok(b) => b,
        Err(e) => {
            res.status_code(StatusCode::BAD_REQUEST);
            res.render(Json(serde_json::json!({
                "success": false,
                "message": format!("请求解析失败: {}", e)
            })));
            return;
        }
    };

    match skill_ops::permanent_delete_skill(&db, &skills_dir, &body.skill_name) {
        Ok(msg) => {
            res.render(Json(serde_json::json!({
                "success": true,
                "message": msg
            })));
        }
        Err(e) => {
            res.status_code(StatusCode::INTERNAL_SERVER_ERROR);
            res.render(Json(serde_json::json!({
                "success": false,
                "message": e
            })));
        }
    }
}

/// 启用 skill
#[handler]
pub async fn enable_skill(req: &mut Request, depot: &mut Depot, res: &mut Response) {
    let db = depot.get_typed::<Arc<SkillDb>>().unwrap();

    let body: EnableRequest = match req.parse_json().await {
        Ok(b) => b,
        Err(e) => {
            res.status_code(StatusCode::BAD_REQUEST);
            res.render(Json(serde_json::json!({
                "success": false,
                "message": format!("请求解析失败: {}", e)
            })));
            return;
        }
    };

    match skill_ops::enable_skill(&db, &body.skill_name) {
        Ok(msg) => {
            res.render(Json(serde_json::json!({
                "success": true,
                "message": msg
            })));
        }
        Err(e) => {
            res.status_code(StatusCode::INTERNAL_SERVER_ERROR);
            res.render(Json(serde_json::json!({
                "success": false,
                "message": e
            })));
        }
    }
}

/// 禁用 skill
#[handler]
pub async fn disable_skill(req: &mut Request, depot: &mut Depot, res: &mut Response) {
    let db = depot.get_typed::<Arc<SkillDb>>().unwrap();

    let body: EnableRequest = match req.parse_json().await {
        Ok(b) => b,
        Err(e) => {
            res.status_code(StatusCode::BAD_REQUEST);
            res.render(Json(serde_json::json!({
                "success": false,
                "message": format!("请求解析失败: {}", e)
            })));
            return;
        }
    };

    match skill_ops::disable_skill(&db, &body.skill_name) {
        Ok(msg) => {
            res.render(Json(serde_json::json!({
                "success": true,
                "message": msg
            })));
        }
        Err(e) => {
            res.status_code(StatusCode::INTERNAL_SERVER_ERROR);
            res.render(Json(serde_json::json!({
                "success": false,
                "message": e
            })));
        }
    }
}
