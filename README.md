# AI 技能百宝箱 (AISkillBox)

[English](#english) | 中文

自动注册和管理 AI Skill 的 MCP 服务。

## 界面预览

![桌面GUI](docs/AISkillBox.png)

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

---

# English

Auto-register and manage AI Skills via MCP service.

## Screenshot

![Desktop GUI](docs/AISkillBox.png)

## Features

1. **Auto-register Skills**
   - Scan skills directory
   - Parse SKILL.md YAML front matter
   - Auto-register as MCP tools

2. **Manage Skills**
   - List registered skills
   - Enable/Disable skills
   - Delete/Restore (trash)
   - Search & filter

3. **Multiple Management Options**
   - AI chat management (recommended): manage via Cursor, Claude, etc.
   - Desktop GUI: `AISkillBox.exe`
   - Web admin: `http://127.0.0.1:10882/web-admin/`

## Files

| File | Description |
|------|-------------|
| `AISkillBox-mcp.exe` | MCP server (port 10881) + Web Admin (port 10882) |
| `AISkillBox.exe` | Desktop management client |

## Directory Structure

```
AISkillBox/
├── skills/                    # skills directory
│   ├── easyclick-android/
│   │   └── SKILL.md
│   ├── easyclick-ios/
│   │   └── SKILL.md
│   └── ...
├── skill-trash/               # trash
├── config.toml                # config file
├── skills.db                  # SQLite database
├── public/                    # usage page
└── web-admin/                 # Web admin panel
```

## Configuration

```toml
listen_addr = "127.0.0.1:10882"
mcp_listen_addr = "127.0.0.1:10881"

[mcp]
tools_path = "tools"
skills_dir = "skills"

[log]
level = "info"
```

## MCP Tools

| Tool | Description |
|------|-------------|
| `list_skills` | List all skills |
| `search_skills` | Search skills |
| `enable_skill` | Enable a skill |
| `disable_skill` | Disable a skill |
| `delete_skill` | Delete skill (move to trash) |
| `restore_skill` | Restore from trash |
| `list_trash` | View trash |
| `refresh_skills` | Refresh skill list |
| `migrate_skills` | Migration guide |

## AI Chat Setup

Add to Cursor's `mcp.json` or Claude's `claude_desktop_config.json`:

```json
{
  "mcpServers": {
    "skill-manager": {
      "url": "http://127.0.0.1:10881/mcp"
    }
  }
}
```

Once configured, just ask the AI: "list all my skills".

---

## 开源协议与版权

Copyright (c) Mr_老鬼. All rights reserved.

本项目采用自定义版权协议，允许自由使用、修改和分发，但 **二开或衍生作品必须保留原始版权声明和出处链接**。

---

## License

Copyright (c) Mr_老鬼. All rights reserved.

This project allows free use, modification, and distribution, but **derivative works must retain the original copyright notice and attribution link**.

---

**Author: Mr_老鬼** | QQ: 1156346325 | B站: Mr_老鬼 | Website: https://www.junjiestudio.top
