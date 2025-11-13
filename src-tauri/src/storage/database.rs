use rusqlite::{Connection, OpenFlags};
use std::path::Path;
use std::sync::{Arc, Mutex};
use super::{StorageError, StorageResult};
use super::schema::{INIT_SCHEMA, PRAGMAS};

/// Thread-safe database connection wrapper
#[derive(Clone)]
pub struct Database {
    conn: Arc<Mutex<Connection>>,
}

impl Database {
    /// Create a new database connection
    pub fn new<P: AsRef<Path>>(path: P) -> StorageResult<Self> {
        let conn = Connection::open_with_flags(
            path,
            OpenFlags::SQLITE_OPEN_READ_WRITE
                | OpenFlags::SQLITE_OPEN_CREATE
                | OpenFlags::SQLITE_OPEN_NO_MUTEX,
        )?;

        // Apply performance pragmas
        conn.execute_batch(PRAGMAS)?;

        // Initialize schema
        conn.execute_batch(INIT_SCHEMA)?;

        Ok(Self {
            conn: Arc::new(Mutex::new(conn)),
        })
    }

    /// Create an in-memory database (for testing)
    pub fn in_memory() -> StorageResult<Self> {
        let conn = Connection::open_in_memory()?;
        conn.execute_batch(PRAGMAS)?;
        conn.execute_batch(INIT_SCHEMA)?;

        Ok(Self {
            conn: Arc::new(Mutex::new(conn)),
        })
    }

    /// Execute a closure with read access to the connection
    pub fn read<F, R>(&self, f: F) -> StorageResult<R>
    where
        F: FnOnce(&Connection) -> StorageResult<R>,
    {
        let conn = self.conn.lock().map_err(|_| {
            StorageError::InvalidOperation("Failed to acquire database lock".to_string())
        })?;
        f(&conn)
    }

    /// Execute a closure with write access to the connection
    pub fn write<F, R>(&self, f: F) -> StorageResult<R>
    where
        F: FnOnce(&Connection) -> StorageResult<R>,
    {
        let conn = self.conn.lock().map_err(|_| {
            StorageError::InvalidOperation("Failed to acquire database lock".to_string())
        })?;
        f(&conn)
    }

    /// Execute a transaction
    pub fn transaction<F, R>(&self, f: F) -> StorageResult<R>
    where
        F: FnOnce(&rusqlite::Transaction) -> StorageResult<R>,
    {
        let mut conn = self.conn.lock().map_err(|_| {
            StorageError::InvalidOperation("Failed to acquire database lock".to_string())
        })?;
        let tx = conn.transaction()?;
        let result = f(&tx)?;
        tx.commit()?;
        Ok(result)
    }

    /// Optimize the database
    pub fn optimize(&self) -> StorageResult<()> {
        self.write(|conn| {
            conn.execute_batch("PRAGMA optimize;")?;
            conn.execute_batch("VACUUM;")?;
            Ok(())
        })
    }

    /// Get database statistics
    pub fn get_stats(&self) -> StorageResult<DatabaseStats> {
        self.read(|conn| {
            let note_count: i64 = conn.query_row(
                "SELECT COUNT(*) FROM notes WHERE is_archived = 0",
                [],
                |row| row.get(0),
            )?;

            let archived_count: i64 = conn.query_row(
                "SELECT COUNT(*) FROM notes WHERE is_archived = 1",
                [],
                |row| row.get(0),
            )?;

            let tag_count: i64 = conn.query_row(
                "SELECT COUNT(*) FROM tags",
                [],
                |row| row.get(0),
            )?;

            let link_count: i64 = conn.query_row(
                "SELECT COUNT(*) FROM note_links",
                [],
                |row| row.get(0),
            )?;

            let node_count: i64 = conn.query_row(
                "SELECT COUNT(*) FROM graph_nodes",
                [],
                |row| row.get(0),
            )?;

            let edge_count: i64 = conn.query_row(
                "SELECT COUNT(*) FROM graph_edges",
                [],
                |row| row.get(0),
            )?;

            Ok(DatabaseStats {
                note_count: note_count as usize,
                archived_count: archived_count as usize,
                tag_count: tag_count as usize,
                link_count: link_count as usize,
                node_count: node_count as usize,
                edge_count: edge_count as usize,
            })
        })
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DatabaseStats {
    pub note_count: usize,
    pub archived_count: usize,
    pub tag_count: usize,
    pub link_count: usize,
    pub node_count: usize,
    pub edge_count: usize,
}
