# AISkillBox

> Skill management tool for ordinary users, making AI skill management as easy as using a phone.

## What is this?

You have many AI skill files scattered across various folders, hard to find, manage, and easy to lose.

**AISkillBox helps you:**
- One-click scan all skills
- Visual management (enable/disable/delete)
- Deleted by mistake? One-click restore from trash
- Can't find it? Quick search

## Who is this for?

| User | Scenario |
|------|----------|
| AI beginners | Just started with Cursor/Claude, don't know how to manage skills |
| Skill collectors | Collected dozens of skills, can't find or manage them |
| Regular users | Don't want to code, just want to manage skills with clicks |

## 3 Steps to Start

> ✅ No Python/Node installation required, no code changes needed, just double-click to run

1. **Download and extract**
2. **Double-click** `AISkillBox-mcp.exe`
3. **Configure connection** (add MCP in any AI editor)

That's it.

## Screenshot

![Desktop GUI](docs/AISkillBox.png)

## What can it do?

### Scan Skills
```
Your skills/ folder
    ↓ Auto scan
AISkillBox lists all skills
    ↓ Auto register
AI assistants can use these skills
```

### Manage Skills

| Action | Description |
|--------|-------------|
| Enable | Let AI assistants use this skill |
| Disable | Temporarily prevent AI from using this skill |
| Delete | Don't want it? Goes to trash first (won't be lost, AI can't access it, safe to use) |
| Restore | Deleted by mistake? One-click restore from trash |
| Search | Quick find by keyword/tag |

### Three Management Methods

| Method | For whom |
|--------|----------|
| **AI Chat** (Recommended) | Just tell AI "list all skills" |
| **Desktop Client** | Double-click to open, click with mouse |
| **Web Admin** | Open in browser, manage anywhere |

## How to configure?

Add to any AI editor (Cursor/Claude/Zed/Windsurf etc.) `mcp.json`:

```json
{
  "mcpServers": {
    "skill-manager": {
      "url": "http://127.0.0.1:10881/mcp"
    }
  }
}
```

After configuration, tell AI "list all skills" to test.

## What's different from other tools?

| Comparison | Other Tools | **AISkillBox** |
|------------|-------------|----------------|
| Requires coding | ✅ Need to edit code | ❌ Click with mouse |
| Visual | ❌ Command line | ✅ GUI + Web |
| Trash | ❌ Deleted = gone forever | ✅ Restore from trash |
| Search | ❌ Manual find | ✅ Keyword search |
| Batch ops | ❌ One by one | ✅ Batch manage |
| Cross-editor | ❌ Locked to single editor | ✅ One config, works everywhere |

## Program Files

| File | Description |
|------|-------------|
| `AISkillBox-mcp.exe` | Main program (Port 10881 + Web Admin 10882) |
| `AISkillBox.exe` | Desktop management client |

## Directory Structure

```
AISkillBox/
├── skills/           # Your skills folder
├── skill-trash/      # Trash (deleted skills)
├── skills.db         # Database (auto managed)
├── config.toml       # Config file (一般不用改)
└── web-admin/        # Web admin panel
```

---

<details>
<summary>Technical Details (click to expand)</summary>

### MCP Tools

| Tool | Description |
|------|-------------|
| `list_skills` | List all skills |
| `search_skills` | Search skills |
| `enable_skill` | Enable skill |
| `disable_skill` | Disable skill |
| `delete_skill` | Delete skill (move to trash) |
| `restore_skill` | Restore from trash |
| `refresh_skills` | Refresh skill list |

### Configuration

```toml
listen_addr = "127.0.0.1:10882"
mcp_listen_addr = "127.0.0.1:10881"
```

### Design Philosophy

1. **Less is more**: Only implement what's truly needed, no feature bloat
2. **Distill, don't hoard**: Only keep valuable information, no duplicate storage
3. **Conversational ops**: Manage through AI chat, more efficient than GUI
4. **Local first**: All data stored locally, no external dependencies
5. **Zero dependencies**: Double-click to run, no environment setup needed
6. **Cross-editor compatible**: One config works with Cursor/Claude/Zed and any AI editor

</details>

---

## License

Copyright (c) Mr_老鬼. All rights reserved.

Free to use, modify, and distribute, but **must retain original copyright notice and attribution link**.

For commercial use or secondary development, please contact the author for authorization: QQ 1156346325 / Website https://www.junjiestudio.top

---

## Tech Stack

### Core Technologies

| Technology | Purpose |
|------------|---------|
| **Rust** | Main language, high performance, memory safety |
| **rmcp** | MCP protocol official SDK (compatible with 2026-07 stateless MCP spec) |
| **SQLite** | Local data persistence |
| **Tokio** | Async runtime |
| **Axum** | HTTP framework |
| **egui** | Desktop GUI |

### Architecture

```
┌─────────────────────────────────────────┐
│              AISkillBox                 │
├─────────────────────────────────────────┤
│  MCP Server (Port 10881)               │
│  └─ rmcp SDK + Streamable HTTP         │
├─────────────────────────────────────────┤
│  Web Admin (Port 10882)                 │
│  └─ Axum + Static file serving         │
├─────────────────────────────────────────┤
│  Desktop GUI                            │
│  └─ egui + eframe                      │
├─────────────────────────────────────────┤
│  Data Layer                             │
│  └─ SQLite (skills.db)                 │
└─────────────────────────────────────────┘
```

### Build

```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Clone project
git clone https://github.com/Mr-老鬼/AISkillBox.git
cd AISkillBox

# Build Release
cargo build --release

# Output location
# target/release/AISkillBox-mcp.exe
# target/release/AISkillBox.exe
```

### Dependencies

```toml
[dependencies]
rmcp = "2.0"           # MCP protocol (compatible with 2026-07 stateless MCP spec)
tokio = "1"            # Async runtime
axum = "0.8"           # HTTP framework
rusqlite = "0.37"      # SQLite
serde = "1"            # Serialization
figment = "0.10"       # Configuration
tracing = "0.1"        # Logging
walkdir = "2"          # Directory traversal
yaml-rust2 = "0.11"    # YAML parsing
eframe = "0.33"        # GUI framework
```

---

**Author: Mr_老鬼** | QQ: 1156346325 | B站: Mr_老鬼 | Website: https://www.junjiestudio.top
