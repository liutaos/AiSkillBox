// Copyright (c) Mr_老鬼. All rights reserved.
// https://www.junjiestudio.top

use salvo::affix_state;
use salvo::prelude::*;
use salvo::serve_static::StaticDir;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;

use super::handlers::{config, service, skill};
use crate::db::SkillDb;

pub fn create_router(
    db: Arc<SkillDb>,
    skills_dir: String,
    exe_dir: std::path::PathBuf,
    reload_flag: Arc<AtomicBool>,
) -> Router {
    let public_dir = exe_dir.join("public");
    let web_admin_dir = exe_dir.join("web-admin");

    Router::new()
        .push(
            Router::with_path("api/admin")
                .hoop(affix_state::inject(db))
                .hoop(affix_state::inject(skills_dir))
                .hoop(affix_state::inject(exe_dir))
                .hoop(affix_state::inject(reload_flag))
                .push(Router::with_path("skills").get(skill::list_skills))
                .push(Router::with_path("trash").get(skill::list_trash))
                .push(Router::with_path("search").post(skill::search_skills))
                .push(Router::with_path("delete").post(skill::delete_skill))
                .push(Router::with_path("restore").post(skill::restore_skill))
                .push(Router::with_path("permanent_delete").post(skill::permanent_delete_skill))
                .push(Router::with_path("enable").post(skill::enable_skill))
                .push(Router::with_path("disable").post(skill::disable_skill))
                .push(Router::with_path("start").post(service::start_service))
                .push(Router::with_path("stop").post(service::stop_service))
                .push(Router::with_path("restart").post(service::restart_service))
                .push(Router::with_path("status").get(service::check_status))
                .push(Router::with_path("refresh").post(service::refresh_skills))
                .push(Router::with_path("config").get(config::get_config))
        )
        .push(
            Router::with_path("web-admin/{*path}").get(
                StaticDir::new([web_admin_dir]).fallback("index.html")
            )
        )
        .push(
            Router::with_path("{*path}").get(
                StaticDir::new([public_dir]).fallback("index.html")
            )
        )
}

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
