use anyhow::Result;
use rusqlite::{Connection, params};
use std::path::Path;
use std::sync::Mutex;

/// Thread-safe wrapper around the SQLite connection.
pub struct Database {
    conn: Mutex<Connection>,
}

impl Database {
    /// Opens (or creates) the SQLite database at `memory/chronodrake.db`
    /// relative to the given root path, and initializes all tables.
    pub fn open(root_path: &str) -> Result<Self> {
        let db_dir = Path::new(root_path).join("memory");
        std::fs::create_dir_all(&db_dir)?;
        let db_path = db_dir.join("chronodrake.db");
        let conn = Connection::open(&db_path)?;
        let db = Database {
            conn: Mutex::new(conn),
        };
        db.initialize_tables()?;
        Ok(db)
    }

    /// Creates all tables if they do not exist.
    fn initialize_tables(&self) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute_batch(
            "
            CREATE TABLE IF NOT EXISTS projects (
                id          TEXT PRIMARY KEY,
                name        TEXT NOT NULL,
                project_type TEXT NOT NULL,
                created_at  TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS scans (
                id              TEXT PRIMARY KEY,
                project_id      TEXT NOT NULL,
                timestamp       TEXT NOT NULL,
                maturity_score  INTEGER NOT NULL,
                security_score  INTEGER NOT NULL,
                architecture_score INTEGER NOT NULL DEFAULT 0,
                scalability_score  INTEGER NOT NULL DEFAULT 0,
                testing_score      INTEGER NOT NULL DEFAULT 0,
                documentation_score INTEGER NOT NULL DEFAULT 0,
                tech_count      INTEGER NOT NULL DEFAULT 0,
                dep_count       INTEGER NOT NULL DEFAULT 0,
                issue_count     INTEGER NOT NULL DEFAULT 0,
                FOREIGN KEY (project_id) REFERENCES projects(id)
            );

            CREATE TABLE IF NOT EXISTS snapshots (
                id              TEXT PRIMARY KEY,
                scan_id         TEXT NOT NULL,
                snapshot_path   TEXT NOT NULL,
                FOREIGN KEY (scan_id) REFERENCES scans(id)
            );

            CREATE TABLE IF NOT EXISTS decisions (
                id          TEXT PRIMARY KEY,
                title       TEXT NOT NULL,
                reason      TEXT NOT NULL,
                timestamp   TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS timeline (
                id          TEXT PRIMARY KEY,
                event       TEXT NOT NULL,
                impact      TEXT NOT NULL,
                timestamp   TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS evolution (
                id          TEXT PRIMARY KEY,
                metric      TEXT NOT NULL,
                old_value   REAL NOT NULL,
                new_value   REAL NOT NULL,
                timestamp   TEXT NOT NULL
            );
            CREATE TABLE IF NOT EXISTS module_relationships (
                id          TEXT PRIMARY KEY,
                source      TEXT NOT NULL,
                target      TEXT NOT NULL,
                rel_type    TEXT NOT NULL,
                weight      REAL NOT NULL DEFAULT 1.0,
                metadata    TEXT NOT NULL DEFAULT '{}'
            );

            CREATE TABLE IF NOT EXISTS dependency_relationships (
                id          TEXT PRIMARY KEY,
                dep_name    TEXT NOT NULL,
                category    TEXT NOT NULL,
                is_critical INTEGER NOT NULL DEFAULT 0,
                is_security INTEGER NOT NULL DEFAULT 0,
                is_ai       INTEGER NOT NULL DEFAULT 0,
                is_blockchain INTEGER NOT NULL DEFAULT 0,
                is_async    INTEGER NOT NULL DEFAULT 0,
                ecosystem   TEXT NOT NULL,
                version     TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS impact_events (
                id          TEXT PRIMARY KEY,
                source      TEXT NOT NULL,
                target      TEXT NOT NULL,
                impact_type TEXT NOT NULL,
                direction   TEXT NOT NULL,
                description TEXT NOT NULL,
                timestamp   TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS knowledge_nodes (
                id          TEXT PRIMARY KEY,
                node_type   TEXT NOT NULL,
                name        TEXT NOT NULL,
                category    TEXT NOT NULL,
                description TEXT NOT NULL DEFAULT '',
                connections INTEGER NOT NULL DEFAULT 0,
                weight      REAL NOT NULL DEFAULT 1.0
            );
            "
        )?;
        Ok(())
    }

    // ── Module Relationships ────────────────────────────────────

    pub fn insert_module_relationship(
        &self, id: &str, source: &str, target: &str, rel_type: &str,
        weight: f64, metadata: &str,
    ) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT OR REPLACE INTO module_relationships (id, source, target, rel_type, weight, metadata)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            rusqlite::params![id, source, target, rel_type, weight, metadata],
        )?;
        Ok(())
    }

    pub fn get_all_module_relationships(&self) -> Result<Vec<ModuleRelationshipRow>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, source, target, rel_type, weight, metadata FROM module_relationships ORDER BY weight DESC",
        )?;
        let rows = stmt.query_map([], |row| {
            Ok(ModuleRelationshipRow {
                id: row.get(0)?,
                source: row.get(1)?,
                target: row.get(2)?,
                rel_type: row.get(3)?,
                weight: row.get(4)?,
                metadata: row.get(5)?,
            })
        })?;
        let mut result = Vec::new();
        for row in rows {
            result.push(row?);
        }
        Ok(result)
    }

    pub fn get_relationships_by_source(&self, source: &str) -> Result<Vec<ModuleRelationshipRow>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, source, target, rel_type, weight, metadata FROM module_relationships
             WHERE source = ?1 ORDER BY weight DESC",
        )?;
        let rows = stmt.query_map(rusqlite::params![source], |row| {
            Ok(ModuleRelationshipRow {
                id: row.get(0)?,
                source: row.get(1)?,
                target: row.get(2)?,
                rel_type: row.get(3)?,
                weight: row.get(4)?,
                metadata: row.get(5)?,
            })
        })?;
        let mut result = Vec::new();
        for row in rows {
            result.push(row?);
        }
        Ok(result)
    }

    // ── Dependency Relationships ────────────────────────────────

    pub fn insert_dependency_relationship(
        &self, id: &str, dep_name: &str, category: &str,
        is_critical: bool, is_security: bool, is_ai: bool,
        is_blockchain: bool, is_async: bool,
        ecosystem: &str, version: &str,
    ) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT OR REPLACE INTO dependency_relationships (id, dep_name, category, is_critical, is_security, is_ai, is_blockchain, is_async, ecosystem, version)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
            rusqlite::params![id, dep_name, category, is_critical as i32, is_security as i32, is_ai as i32, is_blockchain as i32, is_async as i32, ecosystem, version],
        )?;
        Ok(())
    }

    pub fn get_all_dependency_relationships(&self) -> Result<Vec<DependencyRelationshipRow>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, dep_name, category, is_critical, is_security, is_ai, is_blockchain, is_async, ecosystem, version
             FROM dependency_relationships ORDER BY dep_name",
        )?;
        let rows = stmt.query_map([], |row| {
            Ok(DependencyRelationshipRow {
                id: row.get(0)?,
                dep_name: row.get(1)?,
                category: row.get(2)?,
                is_critical: row.get::<_, i32>(3)? != 0,
                is_security: row.get::<_, i32>(4)? != 0,
                is_ai: row.get::<_, i32>(5)? != 0,
                is_blockchain: row.get::<_, i32>(6)? != 0,
                is_async: row.get::<_, i32>(7)? != 0,
                ecosystem: row.get(8)?,
                version: row.get(9)?,
            })
        })?;
        let mut result = Vec::new();
        for row in rows {
            result.push(row?);
        }
        Ok(result)
    }

    pub fn get_critical_dependencies(&self) -> Result<Vec<DependencyRelationshipRow>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, dep_name, category, is_critical, is_security, is_ai, is_blockchain, is_async, ecosystem, version
             FROM dependency_relationships WHERE is_critical = 1 ORDER BY dep_name",
        )?;
        let rows = stmt.query_map([], |row| {
            Ok(DependencyRelationshipRow {
                id: row.get(0)?,
                dep_name: row.get(1)?,
                category: row.get(2)?,
                is_critical: row.get::<_, i32>(3)? != 0,
                is_security: row.get::<_, i32>(4)? != 0,
                is_ai: row.get::<_, i32>(5)? != 0,
                is_blockchain: row.get::<_, i32>(6)? != 0,
                is_async: row.get::<_, i32>(7)? != 0,
                ecosystem: row.get(8)?,
                version: row.get(9)?,
            })
        })?;
        let mut result = Vec::new();
        for row in rows {
            result.push(row?);
        }
        Ok(result)
    }

    // ── Impact Events ───────────────────────────────────────────

    pub fn insert_impact_event(
        &self, id: &str, source: &str, target: &str,
        impact_type: &str, direction: &str, description: &str, timestamp: &str,
    ) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT OR REPLACE INTO impact_events (id, source, target, impact_type, direction, description, timestamp)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            rusqlite::params![id, source, target, impact_type, direction, description, timestamp],
        )?;
        Ok(())
    }

    pub fn get_all_impact_events(&self) -> Result<Vec<ImpactEventRow>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, source, target, impact_type, direction, description, timestamp
             FROM impact_events ORDER BY timestamp DESC",
        )?;
        let rows = stmt.query_map([], |row| {
            Ok(ImpactEventRow {
                id: row.get(0)?,
                source: row.get(1)?,
                target: row.get(2)?,
                impact_type: row.get(3)?,
                direction: row.get(4)?,
                description: row.get(5)?,
                timestamp: row.get(6)?,
            })
        })?;
        let mut result = Vec::new();
        for row in rows {
            result.push(row?);
        }
        Ok(result)
    }

    pub fn get_impacts_by_target(&self, target: &str) -> Result<Vec<ImpactEventRow>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, source, target, impact_type, direction, description, timestamp
             FROM impact_events WHERE target = ?1 ORDER BY timestamp DESC",
        )?;
        let rows = stmt.query_map(rusqlite::params![target], |row| {
            Ok(ImpactEventRow {
                id: row.get(0)?,
                source: row.get(1)?,
                target: row.get(2)?,
                impact_type: row.get(3)?,
                direction: row.get(4)?,
                description: row.get(5)?,
                timestamp: row.get(6)?,
            })
        })?;
        let mut result = Vec::new();
        for row in rows {
            result.push(row?);
        }
        Ok(result)
    }

    // ── Knowledge Nodes ─────────────────────────────────────────

    pub fn insert_knowledge_node(
        &self, id: &str, node_type: &str, name: &str,
        category: &str, description: &str, connections: i32, weight: f64,
    ) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT OR REPLACE INTO knowledge_nodes (id, node_type, name, category, description, connections, weight)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            rusqlite::params![id, node_type, name, category, description, connections, weight],
        )?;
        Ok(())
    }

    pub fn get_all_knowledge_nodes(&self) -> Result<Vec<KnowledgeNodeRow>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, node_type, name, category, description, connections, weight
             FROM knowledge_nodes ORDER BY connections DESC, weight DESC",
        )?;
        let rows = stmt.query_map([], |row| {
            Ok(KnowledgeNodeRow {
                id: row.get(0)?,
                node_type: row.get(1)?,
                name: row.get(2)?,
                category: row.get(3)?,
                description: row.get(4)?,
                connections: row.get(5)?,
                weight: row.get(6)?,
            })
        })?;
        let mut result = Vec::new();
        for row in rows {
            result.push(row?);
        }
        Ok(result)
    }

    pub fn get_nodes_by_type(&self, node_type: &str) -> Result<Vec<KnowledgeNodeRow>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, node_type, name, category, description, connections, weight
             FROM knowledge_nodes WHERE node_type = ?1 ORDER BY connections DESC",
        )?;
        let rows = stmt.query_map(rusqlite::params![node_type], |row| {
            Ok(KnowledgeNodeRow {
                id: row.get(0)?,
                node_type: row.get(1)?,
                name: row.get(2)?,
                category: row.get(3)?,
                description: row.get(4)?,
                connections: row.get(5)?,
                weight: row.get(6)?,
            })
        })?;
        let mut result = Vec::new();
        for row in rows {
            result.push(row?);
        }
        Ok(result)
    }

    pub fn clear_knowledge_graph(&self) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute("DELETE FROM module_relationships", [])?;
        conn.execute("DELETE FROM dependency_relationships", [])?;
        conn.execute("DELETE FROM impact_events", [])?;
        conn.execute("DELETE FROM knowledge_nodes", [])?;
        Ok(())
    }

    // ── Projects ──────────────────────────────────────────────

    pub fn upsert_project(&self, id: &str, name: &str, project_type: &str, created_at: &str) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO projects (id, name, project_type, created_at)
             VALUES (?1, ?2, ?3, ?4)
             ON CONFLICT(id) DO UPDATE SET
                name = excluded.name,
                project_type = excluded.project_type",
            params![id, name, project_type, created_at],
        )?;
        Ok(())
    }

    pub fn get_project_id(&self, name: &str) -> Result<Option<String>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare("SELECT id FROM projects WHERE name = ?1 LIMIT 1")?;
        let mut rows = stmt.query(params![name])?;
        match rows.next()? {
            Some(row) => Ok(Some(row.get(0)?)),
            None => Ok(None),
        }
    }

    // ── Scans ─────────────────────────────────────────────────

    pub fn insert_scan(
        &self,
        id: &str,
        project_id: &str,
        timestamp: &str,
        maturity_score: i32,
        security_score: i32,
        architecture_score: i32,
        scalability_score: i32,
        testing_score: i32,
        documentation_score: i32,
        tech_count: i32,
        dep_count: i32,
        issue_count: i32,
    ) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO scans (id, project_id, timestamp, maturity_score, security_score,
             architecture_score, scalability_score, testing_score, documentation_score,
             tech_count, dep_count, issue_count)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
            params![
                id, project_id, timestamp, maturity_score, security_score,
                architecture_score, scalability_score, testing_score, documentation_score,
                tech_count, dep_count, issue_count,
            ],
        )?;
        Ok(())
    }

    pub fn get_previous_scan_scores(&self, project_id: &str) -> Result<Option<PreviousScores>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT maturity_score, security_score, architecture_score, scalability_score,
                    testing_score, documentation_score, tech_count, dep_count, issue_count, timestamp
             FROM scans WHERE project_id = ?1 ORDER BY timestamp DESC LIMIT 1 OFFSET 1",
        )?;
        let mut rows = stmt.query(params![project_id])?;
        match rows.next()? {
            Some(row) => Ok(Some(PreviousScores {
                maturity_score: row.get(0)?,
                security_score: row.get(1)?,
                architecture_score: row.get(2)?,
                scalability_score: row.get(3)?,
                testing_score: row.get(4)?,
                documentation_score: row.get(5)?,
                tech_count: row.get(6)?,
                dep_count: row.get(7)?,
                issue_count: row.get(8)?,
                timestamp: row.get(9)?,
            })),
            None => Ok(None),
        }
    }

    pub fn get_scan_count(&self, project_id: &str) -> Result<i32> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare("SELECT COUNT(*) FROM scans WHERE project_id = ?1")?;
        let count: i32 = stmt.query_row(params![project_id], |row| row.get(0))?;
        Ok(count)
    }

    // ── Snapshots ─────────────────────────────────────────────

    pub fn insert_snapshot(&self, id: &str, scan_id: &str, snapshot_path: &str) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO snapshots (id, scan_id, snapshot_path) VALUES (?1, ?2, ?3)",
            params![id, scan_id, snapshot_path],
        )?;
        Ok(())
    }

    // ── Decisions ─────────────────────────────────────────────

    pub fn insert_decision(&self, id: &str, title: &str, reason: &str, timestamp: &str) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO decisions (id, title, reason, timestamp) VALUES (?1, ?2, ?3, ?4)",
            params![id, title, reason, timestamp],
        )?;
        Ok(())
    }

    pub fn get_all_decisions(&self) -> Result<Vec<DecisionRow>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, title, reason, timestamp FROM decisions ORDER BY timestamp DESC",
        )?;
        let rows = stmt.query_map([], |row| {
            Ok(DecisionRow {
                id: row.get(0)?,
                title: row.get(1)?,
                reason: row.get(2)?,
                timestamp: row.get(3)?,
            })
        })?;
        let mut result = Vec::new();
        for row in rows {
            result.push(row?);
        }
        Ok(result)
    }

    // ── Timeline ──────────────────────────────────────────────

    pub fn insert_timeline_event(&self, id: &str, event: &str, impact: &str, timestamp: &str) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO timeline (id, event, impact, timestamp) VALUES (?1, ?2, ?3, ?4)",
            params![id, event, impact, timestamp],
        )?;
        Ok(())
    }

    pub fn get_all_timeline_events(&self) -> Result<Vec<TimelineRow>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, event, impact, timestamp FROM timeline ORDER BY timestamp DESC",
        )?;
        let rows = stmt.query_map([], |row| {
            Ok(TimelineRow {
                id: row.get(0)?,
                event: row.get(1)?,
                impact: row.get(2)?,
                timestamp: row.get(3)?,
            })
        })?;
        let mut result = Vec::new();
        for row in rows {
            result.push(row?);
        }
        Ok(result)
    }

    // ── Evolution ─────────────────────────────────────────────

    pub fn insert_evolution(&self, id: &str, metric: &str, old_value: f64, new_value: f64, timestamp: &str) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO evolution (id, metric, old_value, new_value, timestamp) VALUES (?1, ?2, ?3, ?4, ?5)",
            params![id, metric, old_value, new_value, timestamp],
        )?;
        Ok(())
    }

    pub fn get_all_evolution(&self) -> Result<Vec<EvolutionRow>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, metric, old_value, new_value, timestamp FROM evolution ORDER BY timestamp DESC",
        )?;
        let rows = stmt.query_map([], |row| {
            Ok(EvolutionRow {
                id: row.get(0)?,
                metric: row.get(1)?,
                old_value: row.get(2)?,
                new_value: row.get(3)?,
                timestamp: row.get(4)?,
            })
        })?;
        let mut result = Vec::new();
        for row in rows {
            result.push(row?);
        }
        Ok(result)
    }

    pub fn get_evolution_by_metric(&self, metric: &str) -> Result<Vec<EvolutionRow>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, metric, old_value, new_value, timestamp FROM evolution
             WHERE metric = ?1 ORDER BY timestamp DESC",
        )?;
        let rows = stmt.query_map(params![metric], |row| {
            Ok(EvolutionRow {
                id: row.get(0)?,
                metric: row.get(1)?,
                old_value: row.get(2)?,
                new_value: row.get(3)?,
                timestamp: row.get(4)?,
            })
        })?;
        let mut result = Vec::new();
        for row in rows {
            result.push(row?);
        }
        Ok(result)
    }
}

// ── Knowledge Graph row types ──────────────────────────────────

#[derive(Debug, Clone)]
pub struct ModuleRelationshipRow {
    pub id: String,
    pub source: String,
    pub target: String,
    pub rel_type: String,
    pub weight: f64,
    pub metadata: String,
}

#[derive(Debug, Clone)]
pub struct DependencyRelationshipRow {
    pub id: String,
    pub dep_name: String,
    pub category: String,
    pub is_critical: bool,
    pub is_security: bool,
    pub is_ai: bool,
    pub is_blockchain: bool,
    pub is_async: bool,
    pub ecosystem: String,
    pub version: String,
}

#[derive(Debug, Clone)]
pub struct ImpactEventRow {
    pub id: String,
    pub source: String,
    pub target: String,
    pub impact_type: String,
    pub direction: String,
    pub description: String,
    pub timestamp: String,
}

#[derive(Debug, Clone)]
pub struct KnowledgeNodeRow {
    pub id: String,
    pub node_type: String,
    pub name: String,
    pub category: String,
    pub description: String,
    pub connections: i32,
    pub weight: f64,
}

// ── Data row types ────────────────────────────────────────────

pub struct PreviousScores {
    pub maturity_score: i32,
    pub security_score: i32,
    pub architecture_score: i32,
    pub scalability_score: i32,
    pub testing_score: i32,
    pub documentation_score: i32,
    pub tech_count: i32,
    pub dep_count: i32,
    pub issue_count: i32,
    pub timestamp: String,
}

#[derive(Debug, Clone)]
pub struct DecisionRow {
    pub id: String,
    pub title: String,
    pub reason: String,
    pub timestamp: String,
}

#[derive(Debug, Clone)]
pub struct TimelineRow {
    pub id: String,
    pub event: String,
    pub impact: String,
    pub timestamp: String,
}

#[derive(Debug, Clone)]
pub struct EvolutionRow {
    pub id: String,
    pub metric: String,
    pub old_value: f64,
    pub new_value: f64,
    pub timestamp: String,
}
