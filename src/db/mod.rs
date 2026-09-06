// Copyright (c) Mr_老鬼. All rights reserved.
// https://www.junjiestudio.top
// Derivative works must retain this copyright notice.

use rusqlite::{Connection, Result};
use serde::Serialize;
use std::sync::Mutex;

#[derive(Debug, Clone, Serialize)]
pub struct Skill {
    pub name: String,
    pub description: String,
    pub tags: String,
    pub file_path: String,
    pub dir_name: String,
    pub enabled: bool,
    pub deleted: bool,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct Memory {
    pub id: i64,
    pub content: String,
    pub tags: Option<String>,
    pub source: Option<String>,
    pub access_count: i64,
    pub last_accessed_at: Option<String>,
    pub is_archived: bool,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct MemoryStats {
    pub total: i64,
    pub active: i64,
    pub archived: i64,
    pub archive_rate: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct Issue {
    pub id: i64,
    pub issue_id: String,
    pub category: String,
    pub task: Option<String>,
    pub scenario: String,
    pub solution: String,
    pub related_api: Option<String>,
    pub frequency: String,
    pub created_at: String,
}

pub trait SkillStore: Send + Sync {
    fn upsert(&self, name: &str, description: &str, tags: &str, file_path: &str, dir_name: &str) -> Result<()>;
    fn list_all(&self) -> Result<Vec<Skill>>;
    fn list_trash(&self) -> Result<Vec<Skill>>;
    fn search(&self, query: &str, tags: Option<&str>) -> Result<Vec<Skill>>;
    fn set_enabled(&self, name: &str, enabled: bool) -> Result<()>;
    fn soft_delete(&self, name: &str) -> Result<()>;
    fn restore(&self, name: &str) -> Result<()>;
    fn permanent_delete(&self, name: &str) -> Result<()>;
    fn get_by_name(&self, name: &str) -> Result<Option<Skill>>;
}

pub trait MemoryStore: Send + Sync {
    fn remember(&self, content: &str, tags: Option<&str>, source: Option<&str>) -> Result<i64>;
    fn recall(&self, query: &str, tags: Option<Vec<String>>, include_archived: bool, limit: i64) -> Result<Vec<Memory>>;
    fn forget(&self, memory_id: i64) -> Result<()>;
    fn archive(&self, older_than_days: i64) -> Result<i64>;
    fn get_stats(&self) -> Result<MemoryStats>;
}

pub trait IssueStore: Send + Sync {
    fn search_issues(&self, query: &str, task: Option<&str>, category: Option<&str>, limit: i64) -> Result<Vec<Issue>>;
    fn report_issue(&self, category: &str, task: Option<&str>, scenario: &str, solution: &str, related_api: Option<&str>) -> Result<i64>;
}

pub struct SkillDb {
    conn: Mutex<Connection>,
}

impl SkillDb {
    pub fn new(db_path: &str) -> Result<Self, rusqlite::Error> {
        let conn = Connection::open(db_path)?;
        
        conn.execute_batch("PRAGMA journal_mode=WAL;")?;
        conn.execute_batch("PRAGMA busy_timeout=5000;")?;
        
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS skills (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT NOT NULL UNIQUE,
                description TEXT DEFAULT '',
                tags TEXT DEFAULT '[]',
                file_path TEXT DEFAULT '',
                dir_name TEXT NOT NULL,
                enabled INTEGER DEFAULT 1,
                deleted INTEGER DEFAULT 0,
                created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
                updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
            );
            
            CREATE INDEX IF NOT EXISTS idx_skills_name ON skills(name);
            CREATE INDEX IF NOT EXISTS idx_skills_description ON skills(description);

            CREATE TABLE IF NOT EXISTS memories (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                content TEXT NOT NULL,
                tags TEXT,
                source TEXT,
                access_count INTEGER DEFAULT 0,
                last_accessed_at TEXT,
                is_archived INTEGER DEFAULT 0,
                created_at TEXT DEFAULT (datetime('now', 'localtime')),
                updated_at TEXT DEFAULT (datetime('now', 'localtime'))
            );

            CREATE INDEX IF NOT EXISTS idx_memories_created_at ON memories(created_at);
            CREATE INDEX IF NOT EXISTS idx_memories_is_archived ON memories(is_archived);

            CREATE TABLE IF NOT EXISTS common_issues (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                issue_id TEXT NOT NULL UNIQUE,
                category TEXT NOT NULL,
                task TEXT,
                scenario TEXT NOT NULL,
                solution TEXT NOT NULL,
                related_api TEXT,
                frequency TEXT DEFAULT 'low',
                created_at TEXT DEFAULT (datetime('now', 'localtime'))
            );

            CREATE INDEX IF NOT EXISTS idx_common_issues_category ON common_issues(category);
            CREATE INDEX IF NOT EXISTS idx_common_issues_scenario ON common_issues(scenario);
            ",
        )?;
        
        let has_deleted: bool = conn.prepare(
            "SELECT deleted FROM skills LIMIT 1"
        ).is_ok();
        if !has_deleted {
            conn.execute_batch("ALTER TABLE skills ADD COLUMN deleted INTEGER DEFAULT 0;")?;
        }
        
        Ok(SkillDb {
            conn: Mutex::new(conn),
        })
    }

    pub fn get_conn(&self) -> std::sync::MutexGuard<'_, Connection> {
        self.conn.lock().unwrap()
    }
}

fn row_to_skill(row: &rusqlite::Row) -> rusqlite::Result<Skill> {
    Ok(Skill {
        name: row.get(0)?,
        description: row.get(1)?,
        tags: row.get(2)?,
        file_path: row.get(3)?,
        dir_name: row.get(4)?,
        enabled: row.get::<_, i32>(5)? == 1,
        deleted: row.get::<_, i32>(6)? == 1,
        created_at: row.get(7)?,
        updated_at: row.get(8)?,
    })
}

fn row_to_memory(row: &rusqlite::Row) -> rusqlite::Result<Memory> {
    Ok(Memory {
        id: row.get(0)?,
        content: row.get(1)?,
        tags: row.get(2)?,
        source: row.get(3)?,
        access_count: row.get(4)?,
        last_accessed_at: row.get(5)?,
        is_archived: row.get::<_, i32>(6)? == 1,
        created_at: row.get(7)?,
        updated_at: row.get(8)?,
    })
}

fn row_to_issue(row: &rusqlite::Row) -> rusqlite::Result<Issue> {
    Ok(Issue {
        id: row.get(0)?,
        issue_id: row.get(1)?,
        category: row.get(2)?,
        task: row.get(3)?,
        scenario: row.get(4)?,
        solution: row.get(5)?,
        related_api: row.get(6)?,
        frequency: row.get(7)?,
        created_at: row.get(8)?,
    })
}

impl SkillStore for SkillDb {
    fn upsert(&self, name: &str, description: &str, tags: &str, file_path: &str, dir_name: &str) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO skills (name, description, tags, file_path, dir_name, enabled, deleted) 
             VALUES (?1, ?2, ?3, ?4, ?5, 1, 0)
             ON CONFLICT(name) DO UPDATE SET
                 description=excluded.description,
                 tags=excluded.tags,
                 file_path=excluded.file_path,
                 dir_name=excluded.dir_name,
                 deleted=0,
                 updated_at=CURRENT_TIMESTAMP",
            [name, description, tags, file_path, dir_name],
        )?;
        Ok(())
    }

    fn list_all(&self) -> Result<Vec<Skill>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT name, description, tags, file_path, dir_name, enabled, deleted, created_at, updated_at 
             FROM skills WHERE deleted = 0 ORDER BY name"
        )?;
        let skills = stmt.query_map([], row_to_skill)?.collect::<Result<Vec<_>>>()?;
        Ok(skills)
    }

    fn list_trash(&self) -> Result<Vec<Skill>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT name, description, tags, file_path, dir_name, enabled, deleted, created_at, updated_at 
             FROM skills WHERE deleted = 1 ORDER BY name"
        )?;
        let skills = stmt.query_map([], row_to_skill)?.collect::<Result<Vec<_>>>()?;
        Ok(skills)
    }

    fn search(&self, query: &str, tags: Option<&str>) -> Result<Vec<Skill>> {
        let conn = self.conn.lock().unwrap();
        let pattern = format!("%{}%", query);
        
        if let Some(tags) = tags {
            let tag_pattern = format!("%{}%", tags);
            let mut stmt = conn.prepare(
                "SELECT name, description, tags, file_path, dir_name, enabled, deleted, created_at, updated_at 
                 FROM skills WHERE deleted = 0 
                 AND (name LIKE ?1 OR description LIKE ?1 OR tags LIKE ?1) 
                 AND tags LIKE ?2 ORDER BY name"
            )?;
            let skills = stmt.query_map(rusqlite::params![pattern, tag_pattern], row_to_skill)?.collect::<Result<Vec<_>>>()?;
            Ok(skills)
        } else {
            let mut stmt = conn.prepare(
                "SELECT name, description, tags, file_path, dir_name, enabled, deleted, created_at, updated_at 
                 FROM skills WHERE deleted = 0 
                 AND (name LIKE ?1 OR description LIKE ?1 OR tags LIKE ?1) ORDER BY name"
            )?;
            let skills = stmt.query_map(rusqlite::params![pattern], row_to_skill)?.collect::<Result<Vec<_>>>()?;
            Ok(skills)
        }
    }

    fn set_enabled(&self, name: &str, enabled: bool) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        let enabled_int = if enabled { 1 } else { 0 };
        conn.execute(
            "UPDATE skills SET enabled = ?1, updated_at = CURRENT_TIMESTAMP WHERE name = ?2",
            rusqlite::params![enabled_int, name],
        )?;
        Ok(())
    }

    fn soft_delete(&self, name: &str) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "UPDATE skills SET deleted = 1, updated_at = CURRENT_TIMESTAMP WHERE name = ?1",
            [name],
        )?;
        Ok(())
    }

    fn restore(&self, name: &str) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "UPDATE skills SET deleted = 0, updated_at = CURRENT_TIMESTAMP WHERE name = ?1",
            [name],
        )?;
        Ok(())
    }

    fn permanent_delete(&self, name: &str) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "DELETE FROM skills WHERE name = ?1",
            [name],
        )?;
        Ok(())
    }

    fn get_by_name(&self, name: &str) -> Result<Option<Skill>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT name, description, tags, file_path, dir_name, enabled, deleted, created_at, updated_at 
             FROM skills WHERE name = ?1"
        )?;
        let mut rows = stmt.query_map([name], row_to_skill)?;
        match rows.next() {
            Some(row) => Ok(Some(row?)),
            None => Ok(None),
        }
    }
}

fn sanitize_fts_query(query: &str) -> String {
    let special_strs = ["&&", "||"];
    let mut result = query.to_string();
    for s in &special_strs {
        result = result.replace(s, "");
    }
    let special_chars = ['"', '(', ')', ':', '^', '-', '+', '=', '>', '<', '~', '*'];
    for c in special_chars {
        result = result.replace(c, "");
    }
    result
}

impl MemoryStore for SkillDb {
    fn remember(&self, content: &str, tags: Option<&str>, source: Option<&str>) -> Result<i64> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO memories (content, tags, source) VALUES (?1, ?2, ?3)",
            rusqlite::params![content, tags, source],
        )?;
        Ok(conn.last_insert_rowid())
    }

    fn recall(&self, query: &str, tags: Option<Vec<String>>, include_archived: bool, limit: i64) -> Result<Vec<Memory>> {
        let conn = self.conn.lock().unwrap();
        let safe_query = sanitize_fts_query(query);
        
        let mut conditions = vec![];
        if !include_archived {
            conditions.push("is_archived = 0".to_string());
        }
        if !safe_query.is_empty() {
            conditions.push("(content LIKE ?1 OR tags LIKE ?1)".to_string());
        }
        if let Some(ref t) = tags {
            for tag in t {
                conditions.push(format!("tags LIKE '%{}%'", tag.replace('\'', "''")));
            }
        }
        
        let where_clause = if conditions.is_empty() { "1=1".to_string() } else { conditions.join(" AND ") };
        let sql = format!(
            "SELECT id, content, tags, source, access_count, last_accessed_at, is_archived, created_at, updated_at
             FROM memories WHERE {} ORDER BY created_at DESC LIMIT ?2",
            where_clause
        );
        
        let pattern = format!("%{}%", safe_query);
        let mut stmt = conn.prepare(&sql)?;
        let memories = stmt.query_map(rusqlite::params![pattern, limit], row_to_memory)?
            .collect::<Result<Vec<_>>>()?;
        
        // Update access_count and last_accessed_at for retrieved memories
        for mem in &memories {
            conn.execute(
                "UPDATE memories SET access_count = access_count + 1, last_accessed_at = datetime('now', 'localtime') WHERE id = ?1",
                [mem.id],
            )?;
        }
        
        Ok(memories)
    }

    fn forget(&self, memory_id: i64) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute("DELETE FROM memories WHERE id = ?1", [memory_id])?;
        Ok(())
    }

    fn archive(&self, older_than_days: i64) -> Result<i64> {
        let conn = self.conn.lock().unwrap();
        let count = conn.execute(
            "UPDATE memories SET is_archived = 1, updated_at = datetime('now', 'localtime')
             WHERE is_archived = 0 AND created_at < datetime('now', 'localtime', ?1)",
            [format!("-{} days", older_than_days)],
        )?;
        Ok(count as i64)
    }

    fn get_stats(&self) -> Result<MemoryStats> {
        let conn = self.conn.lock().unwrap();
        let total: i64 = conn.query_row("SELECT COUNT(*) FROM memories", [], |r| r.get(0))?;
        let archived: i64 = conn.query_row("SELECT COUNT(*) FROM memories WHERE is_archived = 1", [], |r| r.get(0))?;
        let active = total - archived;
        let archive_rate = if total > 0 { archived as f64 / total as f64 } else { 0.0 };
        
        Ok(MemoryStats {
            total,
            active,
            archived,
            archive_rate,
        })
    }
}

impl IssueStore for SkillDb {
    fn search_issues(&self, query: &str, task: Option<&str>, category: Option<&str>, limit: i64) -> Result<Vec<Issue>> {
        let conn = self.conn.lock().unwrap();
        let safe_query = sanitize_fts_query(query);
        
        let mut conditions = vec![];
        if !safe_query.is_empty() {
            conditions.push("(scenario LIKE ?1 OR solution LIKE ?1 OR issue_id LIKE ?1)".to_string());
        }
        if let Some(t) = task {
            conditions.push(format!("task = '{}'", t.replace('\'', "''")));
        }
        if let Some(c) = category {
            conditions.push(format!("category = '{}'", c.replace('\'', "''")));
        }
        
        let where_clause = if conditions.is_empty() { "1=1".to_string() } else { conditions.join(" AND ") };
        let sql = format!(
            "SELECT id, issue_id, category, task, scenario, solution, related_api, frequency, created_at
             FROM common_issues WHERE {} ORDER BY created_at DESC LIMIT ?2",
            where_clause
        );
        
        let pattern = format!("%{}%", safe_query);
        let mut stmt = conn.prepare(&sql)?;
        let issues = stmt.query_map(rusqlite::params![pattern, limit], row_to_issue)?
            .collect::<Result<Vec<_>>>()?;
        
        Ok(issues)
    }

    fn report_issue(&self, category: &str, task: Option<&str>, scenario: &str, solution: &str, related_api: Option<&str>) -> Result<i64> {
        let conn = self.conn.lock().unwrap();
        let issue_id = uuid::Uuid::new_v4().to_string();
        conn.execute(
            "INSERT INTO common_issues (issue_id, category, task, scenario, solution, related_api) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            rusqlite::params![issue_id, category, task, scenario, solution, related_api],
        )?;
        Ok(conn.last_insert_rowid())
    }
}
