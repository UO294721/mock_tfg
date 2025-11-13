pub mod note_service;
pub mod graph_service;
pub mod search_service;

pub use note_service::*;
pub use graph_service::*;
pub use search_service::*;

use thiserror::Error;
use crate::storage::StorageError;

#[derive(Error, Debug)]
pub enum ServiceError {
    #[error("Storage error: {0}")]
    Storage(#[from] StorageError),

    #[error("Note not found: {0}")]
    NoteNotFound(String),

    #[error("Invalid input: {0}")]
    InvalidInput(String),

    #[error("Operation failed: {0}")]
    OperationFailed(String),
}

pub type ServiceResult<T> = Result<T, ServiceError>;
