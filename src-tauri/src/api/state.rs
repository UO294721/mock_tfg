use crate::services::*;
use crate::storage::*;
use std::sync::Arc;
use parking_lot::RwLock;

/// Application state shared across all Tauri commands
pub struct AppState {
    pub note_service: Arc<RwLock<NoteService>>,
    pub graph_service: Arc<RwLock<GraphService>>,
    pub search_service: Arc<RwLock<SearchService>>,
    pub database: Database,
}

impl AppState {
    pub fn new(database: Database) -> Self {
        let cache = CacheManager::default();
        let note_repository = NoteRepository::new(database.clone());
        let graph_repository = GraphRepository::new(database.clone());

        let note_service = NoteService::new(note_repository.clone(), cache.clone());
        let graph_service = GraphService::new(graph_repository, cache.clone());
        let search_service = SearchService::new(note_repository, cache);

        Self {
            note_service: Arc::new(RwLock::new(note_service)),
            graph_service: Arc::new(RwLock::new(graph_service)),
            search_service: Arc::new(RwLock::new(search_service)),
            database,
        }
    }
}
