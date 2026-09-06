// Copyright (c) Mr_老鬼. All rights reserved.
// https://www.junjiestudio.top
// Derivative works must retain this copyright notice.

use std::sync::Arc;
use crate::db::{MemoryStore, IssueStore, Memory, Issue, MemoryStats};

pub struct MemoryManager {
    db: Arc<dyn MemoryStore>,
    issue_store: Arc<dyn IssueStore>,
}

impl MemoryManager {
    pub fn new(db: Arc<dyn MemoryStore>, issue_store: Arc<dyn IssueStore>) -> Self {
        Self { db, issue_store }
    }

    pub fn remember(&self, content: &str, tags: Option<&str>, source: Option<&str>) -> Result<i64, String> {
        self.db.remember(content, tags, source).map_err(|e| e.to_string())
    }

    pub fn recall(&self, query: &str, tags: Option<Vec<String>>, include_archived: bool, limit: i64) -> Result<Vec<Memory>, String> {
        self.db.recall(query, tags, include_archived, limit).map_err(|e| e.to_string())
    }

    pub fn forget(&self, memory_id: i64) -> Result<(), String> {
        self.db.forget(memory_id).map_err(|e| e.to_string())
    }

    pub fn archive(&self, older_than_days: i64) -> Result<i64, String> {
        self.db.archive(older_than_days).map_err(|e| e.to_string())
    }

    pub fn get_stats(&self) -> Result<MemoryStats, String> {
        self.db.get_stats().map_err(|e| e.to_string())
    }

    pub fn search_issues(&self, query: &str, task: Option<&str>, category: Option<&str>, limit: i64) -> Result<Vec<Issue>, String> {
        self.issue_store.search_issues(query, task, category, limit).map_err(|e| e.to_string())
    }

    pub fn report_issue(&self, category: &str, task: Option<&str>, scenario: &str, solution: &str, related_api: Option<&str>) -> Result<i64, String> {
        self.issue_store.report_issue(category, task, scenario, solution, related_api).map_err(|e| e.to_string())
    }
}
