use std::fs;
use std::path::Path;

/// Integration tests for SQLite persistence layer.
///
/// These tests verify that the database can be created, tables initialized,
/// and data persisted correctly.
#[cfg(test)]
mod sqlite_tests {
    use std::path::Path;

    /// Helper to create a temporary database path.
    fn get_test_db_path(dir: &Path) -> String {
        dir.join("test.db").to_string_lossy().to_string()
    }

    #[test]
    fn test_sqlite_connection() {
        let dir = tempfile::tempdir().unwrap();
        let db_path = get_test_db_path(dir.path());

        let conn = rusqlite::Connection::open(&db_path).unwrap();
        assert!(conn.is_autocommit());
    }

    #[test]
    fn test_sqlite_create_table() {
        let dir = tempfile::tempdir().unwrap();
        let db_path = get_test_db_path(dir.path());
        let conn = rusqlite::Connection::open(&db_path).unwrap();

        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS test_table (
                id INTEGER PRIMARY KEY,
                name TEXT NOT NULL,
                value INTEGER NOT NULL
            );",
        )
        .unwrap();

        // Verify table exists
        let count: i32 = conn
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='test_table'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(count, 1);
    }

    #[test]
    fn test_sqlite_insert_and_query() {
        let dir = tempfile::tempdir().unwrap();
        let db_path = get_test_db_path(dir.path());
        let conn = rusqlite::Connection::open(&db_path).unwrap();

        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS test_data (
                id TEXT PRIMARY KEY,
                value TEXT NOT NULL,
                created_at TEXT NOT NULL
            );",
        )
        .unwrap();

        conn.execute(
            "INSERT INTO test_data (id, value, created_at) VALUES (?1, ?2, ?3)",
            rusqlite::params!["test-1", "hello", "2025-01-01T00:00:00Z"],
        )
        .unwrap();

        let result: String = conn
            .query_row(
                "SELECT value FROM test_data WHERE id = ?1",
                rusqlite::params!["test-1"],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(result, "hello");
    }

    #[test]
    fn test_sqlite_multiple_rows() {
        let dir = tempfile::tempdir().unwrap();
        let db_path = get_test_db_path(dir.path());
        let conn = rusqlite::Connection::open(&db_path).unwrap();

        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS scores (
                id INTEGER PRIMARY KEY,
                name TEXT NOT NULL,
                score INTEGER NOT NULL
            );",
        )
        .unwrap();

        let data = vec![
            ("Alice", 95),
            ("Bob", 87),
            ("Charlie", 92),
        ];

        for (i, (name, score)) in data.iter().enumerate() {
            conn.execute(
                "INSERT INTO scores (id, name, score) VALUES (?1, ?2, ?3)",
                rusqlite::params![i as i32, name, score],
            )
            .unwrap();
        }

        let mut stmt = conn.prepare("SELECT name, score FROM scores ORDER BY score DESC").unwrap();
        let rows: Vec<(String, i32)> = stmt
            .query_map([], |row| {
                Ok((row.get::<_, String>(0)?, row.get::<_, i32>(1)?))
            })
            .unwrap()
            .filter_map(|r| r.ok())
            .collect();

        assert_eq!(rows.len(), 3);
        assert_eq!(rows[0].0, "Alice");
        assert_eq!(rows[0].1, 95);
    }

    #[test]
    fn test_sqlite_transaction_rollback() {
        let dir = tempfile::tempdir().unwrap();
        let db_path = get_test_db_path(dir.path());
        let mut conn = rusqlite::Connection::open(&db_path).unwrap();

        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS tx_test (
                id INTEGER PRIMARY KEY,
                value TEXT NOT NULL
            );",
        )
        .unwrap();

        // Start a transaction and rollback
        {
            let tx = conn.transaction().unwrap();
            tx.execute(
                "INSERT INTO tx_test (id, value) VALUES (?1, ?2)",
                rusqlite::params![1, "should_not_exist"],
            )
            .unwrap();
            // tx is dropped without commit → rollback
        }

        let count: i32 = conn
            .query_row("SELECT COUNT(*) FROM tx_test", [], |row| row.get(0))
            .unwrap();
        assert_eq!(count, 0);
    }

    #[test]
    fn test_sqlite_foreign_key() {
        let dir = tempfile::tempdir().unwrap();
        let db_path = get_test_db_path(dir.path());
        let conn = rusqlite::Connection::open(&db_path).unwrap();

        conn.execute_batch("PRAGMA foreign_keys = ON;").unwrap();
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS parent (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL
            );
            CREATE TABLE IF NOT EXISTS child (
                id TEXT PRIMARY KEY,
                parent_id TEXT NOT NULL,
                FOREIGN KEY (parent_id) REFERENCES parent(id)
            );",
        )
        .unwrap();

        // Insert parent
        conn.execute(
            "INSERT INTO parent (id, name) VALUES (?1, ?2)",
            rusqlite::params!["p1", "parent1"],
        )
        .unwrap();

        // Insert child with valid FK
        conn.execute(
            "INSERT INTO child (id, parent_id) VALUES (?1, ?2)",
            rusqlite::params!["c1", "p1"],
        )
        .unwrap();

        // Verify via JOIN
        let result: String = conn
            .query_row(
                "SELECT child.id FROM child JOIN parent ON child.parent_id = parent.id WHERE parent.name = ?1",
                rusqlite::params!["parent1"],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(result, "c1");
    }
}
