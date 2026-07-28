mod config;
mod mcp;
mod tools;
mod exec;
mod db;
mod management;
mod web_admin;

use tracing::info;
use std::sync::Arc;

#[tokio::main]
async fn main() {
    crate::config::init();
    let cfg = crate::config::get();

    if is_port_in_use(&cfg.mcp_listen_addr) {
        eprintln!("错误: MCP 服务已在运行 (端口 {} 被占用)", cfg.mcp_listen_addr);
        eprintln!("如果需要重启服务，请先停止正在运行的实例");
        std::process::exit(1);
    }

    let env_filter = tracing_subscriber::EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new(&cfg.log.level));

    tracing_subscriber::fmt()
        .with_env_filter(env_filter)
        .init();

    let manager = tools::manager::ToolManager::new(&cfg.mcp.tools_path, &cfg.mcp.skills_dir, &cfg.db_path);
    if let Err(e) = manager.init().await {
        tracing::error!("工具加载失败: {:?}", e);
        eprintln!("工具加载失败: {:?}", e);
        std::process::exit(1);
    }

    let count = manager.count().await;
    info!("加载了 {} 个工具", count);

    let exe_dir = std::env::current_exe()
        .ok()
        .and_then(|exe| exe.parent().map(|p| p.to_path_buf()))
        .unwrap_or_else(|| std::env::current_dir().unwrap_or_default());

    let use_stdio = std::env::args().any(|a| a == "--stdio");

    if use_stdio {
        println!("AI 技能百宝箱 启动中 (stdio 模式)...");
        println!("加载了 {} 个工具", count);

        use rmcp::ServiceExt;
        use rmcp::transport::stdio;

        let handler = mcp::handler::EcMcpHandler::new(manager);
        match handler.serve(stdio()).await {
            Ok(service) => {
                info!("MCP stdio 服务已启动");
                if let Err(e) = service.waiting().await {
                    tracing::error!("MCP stdio 服务异常: {:?}", e);
                }
            }
            Err(e) => {
                tracing::error!("MCP stdio 服务启动失败: {:?}", e);
                eprintln!("MCP stdio 服务启动失败: {:?}", e);
            }
        }
    } else {
        println!("AI 技能百宝箱 启动中 (HTTP 模式)...");
        println!("Web 端口: http://{}", cfg.listen_addr);
        println!("MCP 端口: http://{}", cfg.mcp_listen_addr);
        println!("加载了 {} 个工具", count);

        let handler = mcp::handler::EcMcpHandler::new(manager.clone());
        let mcp_addr = cfg.mcp_listen_addr.clone();

        tokio::spawn(async move {
            if let Err(e) = mcp::service::start_mcp_server(&mcp_addr, handler).await {
                tracing::error!("MCP 服务启动失败: {:?}", e);
            }
        });

        let db = Arc::clone(manager.db());
        let skills_dir = manager.skills_path().to_string();
        let web_addr = cfg.listen_addr.clone();
        let reload_flag = manager.reload_flag();

        tokio::spawn(async move {
            if let Err(e) = web_admin::routes::start_web_admin_server(&web_addr, db, skills_dir, exe_dir, reload_flag).await {
                tracing::error!("Web Admin 服务启动失败: {:?}", e);
            }
        });

        tokio::signal::ctrl_c().await.unwrap();
        info!("收到 Ctrl+C 信号，正在关闭...");
    }
}

fn is_port_in_use(addr: &str) -> bool {
    use std::net::TcpListener;
    TcpListener::bind(addr).is_err()
}
