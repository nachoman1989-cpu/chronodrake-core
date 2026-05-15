use anyhow::Result;
use rusqlite::Connection;
use std::sync::Mutex;

/// Current schema version of the database.
pub const CURRENT_SCHEMA_VERSION: i32 = 2;

/// Migration definitions: each entry is (version, description, SQL).
const MIGRATIONS: &[(i32, &str, &[&str])] = &[
    (
        1,
        "Initial schema: projects, scans, snapshots, decisions, timeline, evolution",
        &[
            "CREATE TABLE IF NOT EXISTS projects (
                id          TEXT PRIMARY KEY,
                name        TEXT NOT NULL,
                project_type TEXT NOT NULL,
                created_at  TEXT NOT NULL
            );",
            "CREATE TABLE IF NOT EXISTS scans (
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
            );",
            "CREATE TABLE IF NOT EXISTS snapshots (
                id              TEXT PRIMARY KEY,
                scan_id         TEXT NOT NULL,
                snapshot_path   TEXT NOT NULL,
                FOREIGN KEY (scan_id) REFERENCES scans(id)
            );",
            "CREATE TABLE IF NOT EXISTS decisions (
                id          TEXT PRIMARY KEY,
                title       TEXT NOT NULL,
                reason      TEXT NOT NULL,
                timestamp   TEXT NOT NULL
            );",
            "CREATE TABLE IF NOT EXISTS timeline (
                id          TEXT PRIMARY KEY,
                event       TEXT NOT NULL,
                impact      TEXT NOT NULL,
                timestamp   TEXT NOT NULL
            );",
            "CREATE TABLE IF NOT EXISTS evolution (
                id          TEXT PRIMARY KEY,
                metric      TEXT NOT NULL,
                old_value   REAL NOT NULL,
                new_value   REAL NOT NULL,
                timestamp   TEXT NOT NULL
            );",
        ],
    ),
    (
        2,
        "Knowledge Graph: module_relationships, dependency_relationships, impact_events, knowledge_nodes",
        &[
            "CREATE TABLE IF NOT EXISTS module_relationships (
                id          TEXT PRIMARY KEY,
                source      TEXT NOT NULL,
                target      TEXT NOT NULL,
                rel_type    TEXT NOT NULL,
                weight      REAL NOT NULL DEFAULT 1.0,
                metadata    TEXT NOT NULL DEFAULT '{}'
            );",
            "CREATE TABLE IF NOT EXISTS dependency_relationships (
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
            );",
            "CREATE TABLE IF NOT EXISTS impact_events (
                id          TEXT PRIMARY KEY,
                source      TEXT NOT NULL,
                target      TEXT NOT NULL,
                impact_type TEXT NOT NULL,
                direction   TEXT NOT NULL,
                description TEXT NOT NULL,
                timestamp   TEXT NOT NULL
            );",
            "CREATE TABLE IF NOT EXISTS knowledge_nodes (
                id          TEXT PRIMARY KEY,
                node_type   TEXT NOT NULL,
                name        TEXT NOT NULL,
                category    TEXT NOT NULL,
                description TEXT NOT NULL DEFAULT '',
                connections INTEGER NOT NULL DEFAULT 0,
                weight      REAL NOT NULL DEFAULT 1.0
            );",
        ],
    ),
];

/// Manages database schema migrations.
pub struct MigrationEngine;

impl MigrationEngine {
    /// Ensures the `schema_version` table exists.
    fn ensure_schema_table(conn: &Connection) -> Result<()> {
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS schema_version (
                version     INTEGER PRIMARY KEY,
                applied_at  TEXT NOT NULL,
                description TEXT NOT NULL
            );",
        )?;
        Ok(())
    }

    /// Returns the current schema version from the database.
    /// Returns 0 if no version has been applied.
    pub fn get_current_version(conn: &Connection) -> Result<i32> {
        Self::ensure_schema_table(conn)?;
        let result: Result<i32, _> = conn.query_row(
            "SELECT COALESCE(MAX(version), 0) FROM schema_version",
            [],
            |row| row.get(0),
        );
        match result {
            Ok(v) => Ok(v),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(0),
            Err(e) => Err(e.into()),
        }
    }

    /// Runs all pending migrations up to `CURRENT_SCHEMA_VERSION`.
    pub fn run_pending(conn: &Connection) -> Result<Vec<String>> {
        let mut applied = Vec::new();
        let current = Self::get_current_version(conn)?;

        for (version, description, statements) in MIGRATIONS {
            if *version > current {
                for stmt in *statements {
                    conn.execute_batch(stmt)?;
                }
                conn.execute(
                    "INSERT INTO schema_version (version, applied_at, description)
                     VALUES (?1, ?2, ?3)",
                    rusqlite::params![
                        version,
                        chrono::Utc::now().to_rfc3339(),
                        description,
                    ],
                )?;
                applied.push(format!("v{}: {}", version, description));
            }
        }

        Ok(applied)
    }

    /// Validates that the database schema is at the expected version.
    pub fn validate(conn: &Connection) -> Result<()> {
        let current = Self::get_current_version(conn)?;
        if current < CURRENT_SCHEMA_VERSION {
            // Run pending migrations automatically
            let applied = Self::run_pending(conn)?;
            if !applied.is_empty() {
                // Migrations were applied — log this
                eprintln!("[migrations] Applied: {}", applied.join(", "));
            }
        }
        Ok(())
    }
}

/// Thread-safe wrapper that runs migrations on a database connection.
pub fn run_migrations(conn: &Mutex<Connection>) -> Result<Vec<String>> {
    let conn = conn.lock().unwrap();
    MigrationEngine::run_pending(&conn)
}
