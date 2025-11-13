pub mod database;
pub mod schema;
pub mod repository;
pub mod cache;

pub use database::*;
pub use repository::*;
pub use cache::*;

use thiserror::Error;

#[derive(Error, Debug)]
pub enum StorageError {
    #[error("Database error: {0}")]
    DatabaseError(#[from] rusqlite::Error),

    #[error("Note not found: {0}")]
    NoteNotFound(String),

    #[error("Graph node not found: {0}")]
    NodeNotFound(String),

    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Invalid operation: {0}")]
    InvalidOperation(String),
}

pub type StorageResult<T> = Result<T, StorageError>;
