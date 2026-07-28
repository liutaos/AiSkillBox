use rmcp::{
    RoleServer,
    ErrorData as McpError,
    handler::server::ServerHandler,
    model::*,
    service::RequestContext,
};

use crate::tools::manager::ToolManager;

#[derive(Clone)]
pub struct EcMcpHandler {
    manager: ToolManager,
}

impl EcMcpHandler {
    pub fn new(manager: ToolManager) -> Self {
        Self { manager }
    }
}

impl ServerHandler for EcMcpHandler {
    fn get_info(&self) -> ServerInfo {
        let capabilities = ServerCapabilities::builder()
            .enable_tools()
            .enable_tool_list_changed()
            .build();

        ServerInfo::new(capabilities)
            .with_server_info(Implementation::new("easyclick-mcp-extension", "0.1.0"))
            .with_instructions("AI 技能百宝箱 - 自动注册和管理 AI Skill 的 MCP 服务。支持从 SKILL.md 解析工具定义，运行时刷新工具列表。")
    }

    fn list_tools<'a>(
        &'a self,
        _request: Option<PaginatedRequestParams>,
        _context: RequestContext<RoleServer>,
    ) -> impl std::future::Future<Output = Result<ListToolsResult, McpError>> + rmcp::service::MaybeSendFuture + 'a
    {
        async move {
            let tools = self.manager.get_tools().await;
            tracing::info!("list_tools 被调用，返回 {} 个工具", tools.len());
            for t in &tools {
                tracing::info!("  工具: {} - {}", t.name, t.description.as_deref().unwrap_or(""));
            }
            Ok(ListToolsResult::with_all_items(tools))
        }
    }

    fn call_tool<'a>(
        &'a self,
        params: CallToolRequestParams,
        _context: RequestContext<RoleServer>,
    ) -> impl std::future::Future<Output = Result<CallToolResult, McpError>> + rmcp::service::MaybeSendFuture + 'a
    {
        async move {
            let tool_name = &*params.name;
            let args = serde_json::Value::Object(
                params.arguments.unwrap_or_default(),
            );

            let executor = self.manager.executor().await;
            let result = executor.execute(tool_name, args).await;

            if let Ok(Some(_count)) = self.manager.check_and_reload().await {
                tracing::info!("工具列表已刷新，当前 {} 个工具", _count);
                let tools = self.manager.get_tools().await;
                let tool_list: Vec<String> = tools.iter().map(|t| {
                    format!("- {} - {}", t.name, t.description.as_deref().unwrap_or(""))
                }).collect();
                return Ok(CallToolResult::success(vec![ContentBlock::text(
                    format!("工具列表已刷新，当前加载 {} 个工具:\n{}", _count, tool_list.join("\n")),
                )]));
            }

            match result {
                Ok(data) => {
                    Ok(CallToolResult::success(vec![ContentBlock::text(
                        serde_json::to_string_pretty(&data).unwrap_or_else(|_| format!("{}", data)),
                    )]))
                }
                Err(e) => {
                    Ok(CallToolResult::error(vec![ContentBlock::text(
                        format!("错误: {}", e),
                    )]))
                }
            }
        }
    }
}
