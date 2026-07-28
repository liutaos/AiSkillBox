use salvo::affix_state;
use salvo::prelude::*;
use salvo::serve_static::StaticDir;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;

use super::handlers::{config, service, skill};
use crate::db::SkillDb;

/// 创建 Web Admin 路由
pub fn create_router(
    db: Arc<SkillDb>,
    skills_dir: String,
    exe_dir: std::path::PathBuf,
    reload_flag: Arc<AtomicBool>,
) -> Router {
    // 静态文件目录
    let public_dir = exe_dir.join("public");
    let web_admin_dir = exe_dir.join("web-admin");

    Router::new()
        // API 路由（优先匹配）
        .push(
            Router::with_path("api/admin")
                .hoop(affix_state::inject(db))
                .hoop(affix_state::inject(skills_dir))
                .hoop(affix_state::inject(exe_dir))
                .hoop(affix_state::inject(reload_flag))
                // 技能管理
                .push(Router::with_path("skills").get(skill::list_skills))
                .push(Router::with_path("trash").get(skill::list_trash))
                .push(Router::with_path("search").post(skill::search_skills))
                .push(Router::with_path("delete").post(skill::delete_skill))
                .push(Router::with_path("restore").post(skill::restore_skill))
                .push(Router::with_path("permanent_delete").post(skill::permanent_delete_skill))
                .push(Router::with_path("enable").post(skill::enable_skill))
                .push(Router::with_path("disable").post(skill::disable_skill))
                // 服务控制
                .push(Router::with_path("start").post(service::start_service))
                .push(Router::with_path("stop").post(service::stop_service))
                .push(Router::with_path("restart").post(service::restart_service))
                .push(Router::with_path("status").get(service::check_status))
                .push(Router::with_path("refresh").post(service::refresh_skills))
                // 配置
                .push(Router::with_path("config").get(config::get_config))
        )
        // Vue 前端：/web-admin/ 前缀
        .push(
            Router::with_path("web-admin/{*path}").get(
                StaticDir::new([web_admin_dir]).fallback("index.html")
            )
        )
        // 其他静态文件（兜底）：public 目录
        .push(
            Router::with_path("{*path}").get(
                StaticDir::new([public_dir]).fallback("index.html")
            )
        )
}

/// 启动 Web Admin 服务
pub async fn start_web_admin_server(
    addr: &str,
    db: Arc<SkillDb>,
    skills_dir: String,
    exe_dir: std::path::PathBuf,
    reload_flag: Arc<AtomicBool>,
) -> Result<(), Box<dyn std::error::Error>> {
    let router = create_router(db, skills_dir, exe_dir, reload_flag);

    tracing::info!("Web Admin 服务监听: http://{}", addr);

    let socket_addr: std::net::SocketAddr = addr.parse()?;
    let acceptor = salvo::conn::TcpListener::new(socket_addr).bind().await;
    Server::new(acceptor).serve(router).await;

    Ok(())
}
