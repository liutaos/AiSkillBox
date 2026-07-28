// Copyright (c) Mr_老鬼. All rights reserved.
// https://www.junjiestudio.top
// Derivative works must retain this copyright notice.

use std::sync::Arc;

use rmcp::transport::streamable_http_server::{
    StreamableHttpServerConfig, StreamableHttpService, session::local::LocalSessionManager,
};

use super::handler::EcMcpHandler;

/// 启动 MCP 协议服务（HTTP 模式）
pub async fn start_mcp_server(addr: &str, handler: EcMcpHandler) -> Result<(), Box<dyn std::error::Error>> {
    let mcp_service = StreamableHttpService::new(
        move || Ok(handler.clone()),
        Arc::new(LocalSessionManager::default()),
        StreamableHttpServerConfig::default(),
    );

    let router = axum::Router::new().nest_service("/mcp", mcp_service);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    tracing::info!("MCP HTTP 服务监听: http://{}", addr);

    axum::serve(listener, router)
        .with_graceful_shutdown(async {
            tokio::signal::ctrl_c().await.ok();
        })
        .await?;

    Ok(())
}
