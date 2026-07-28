use rusqlite::{Connection, Result};
use serde::Serialize;
use std::sync::Mutex;

/// Skill 数据结构
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

/// Skill 数据库操作 trait
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
