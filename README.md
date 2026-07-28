# AI 技能百宝箱 (AISkillBox)

自动注册和管理 AI Skill 的 MCP 服务。

## 核心功能

1. **自动注册 Skill**
   - 扫描 skills 目录
   - 解析 SKILL.md 的 YAML front matter
   - 自动注册为 MCP 工具

2. **管理 Skill**
   - 列出已注册的 Skill
   - 启用/禁用 Skill
   - 删除/恢复（回收站）
   - 搜索过滤

3. **多种管理方式**
   - AI 对话管理（推荐）：通过 Cursor、Claude 等 AI 助手直接管理
   - 桌面 GUI：`AISkillBox.exe`
   - Web 后台：`http://127.0.0.1:10882/web-admin/`

## 程序文件

| 文件 | 说明 |
|------|------|
| `AISkillBox-mcp.exe` | MCP 服务进程（端口 10881）+ Web Admin（端口 10882） |
| `AISkillBox.exe` | 桌面管理客户端 |

## 目录结构

```
AISkillBox/
├── skills/                    # skill 目录
│   ├── easyclick-android/
│   │   └── SKILL.md
│   ├── easyclick-ios/
│   │   └── SKILL.md
│   └── ...
├── skill-trash/               # 回收站
├── config.toml                # 配置文件
├── skills.db                  # SQLite 数据库
├── public/                    # 使用说明页
└── web-admin/                 # Web 管理后台
```

## 配置

```toml
listen_addr = "127.0.0.1:10882"
mcp_listen_addr = "127.0.0.1:10881"

[mcp]
tools_path = "tools"
skills_dir = "skills"

[log]
level = "info"
```

## MCP 工具

| 工具名 | 说明 |
|--------|------|
| `list_skills` | 列出所有 skill |
| `search_skills` | 搜索 skill |
| `enable_skill` | 启用指定 skill |
| `disable_skill` | 禁用指定 skill |
| `delete_skill` | 删除 skill（移入回收站） |
| `restore_skill` | 从回收站恢复 |
| `list_trash` | 查看回收站 |
| `refresh_skills` | 刷新 skill 列表 |
| `migrate_skills` | 迁移指引 |

## AI 对话管理配置

在 Cursor 的 `mcp.json` 或 Claude 的 `claude_desktop_config.json` 中添加：

```json
{
  "mcpServers": {
    "skill-manager": {
      "url": "http://127.0.0.1:10881/mcp"
    }
  }
}
```

配置完成后，直接对 AI 说"帮我列出所有 skill"即可。
