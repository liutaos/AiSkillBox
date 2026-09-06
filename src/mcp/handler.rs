// Copyright (c) Mr_老鬼. All rights reserved.
// https://www.junjiestudio.top
// Derivative works must retain this copyright notice.

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
            .with_instructions(r#"# AI 技能百宝箱

## 记忆系统（必须遵守）
每次开始新任务前：
1. 调用 recall 搜索相关历史记忆
2. 如果找到相关记忆，参考历史方案继续工作
3. 如果没找到，正常执行新任务

每次完成阶段性工作后：
1. 调用 remember 记录关键信息
2. 内容应包含：用户需求、技术方案、关键结果

## 问题处理流程（遇到报错时）
1. 先调用 search_issues 搜索历史解决方案
2. 如果找到匹配问题，参考解决方案
3. 如果没找到，自行分析解决
4. 解决后调用 report_issue 记录新问题，积累问题库

## 问题分类 (category)
安装 / 配置 / 运行 / 兼容性 / 性能 / 功能

## 任务场景 (task)
skill管理 / MCP协议 / 搜索 / 导入 / UI界面 / 数据库

## Skill 管理
- list_skills: 列出所有 skill
- search_skills: 搜索 skill
- enable_skill / disable_skill: 启用/禁用 skill
- delete_skill / restore_skill: 删除/恢复 skill
- refresh_skills: 刷新 skill 列表
"#)
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
