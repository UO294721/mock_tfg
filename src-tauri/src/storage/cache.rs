use lru::LruCache;
use parking_lot::RwLock;
use std::num::NonZeroUsize;
use std::sync::Arc;
use uuid::Uuid;
use crate::domain::*;

/// Multi-level cache system for high-performance note access
pub struct CacheManager {
    note_cache: Arc<RwLock<LruCache<Uuid, Note>>>,
    graph_cache: Arc<RwLock<Option<KnowledgeGraph>>>,
    search_cache: Arc<RwLock<LruCache<String, Vec<Uuid>>>>,
}

impl CacheManager {
    pub fn new(capacity: usize) -> Self {
        Self {
            note_cache: Arc::new(RwLock::new(
                LruCache::new(NonZeroUsize::new(capacity).unwrap()),
            )),
            graph_cache: Arc::new(RwLock::new(None)),
            search_cache: Arc::new(RwLock::new(
                LruCache::new(NonZeroUsize::new(1000).unwrap()),
            )),
        }
    }

    // Note cache operations

    pub fn get_note(&self, id: &Uuid) -> Option<Note> {
        self.note_cache.write().get(id).cloned()
    }

    pub fn put_note(&self, note: Note) {
        self.note_cache.write().put(note.id, note);
    }

    pub fn invalidate_note(&self, id: &Uuid) {
        self.note_cache.write().pop(id);
    }

    pub fn clear_notes(&self) {
        self.note_cache.write().clear();
    }

    // Graph cache operations

    pub fn get_graph(&self) -> Option<KnowledgeGraph> {
        self.graph_cache.read().clone()
    }

    pub fn put_graph(&self, graph: KnowledgeGraph) {
        *self.graph_cache.write() = Some(graph);
    }

    pub fn invalidate_graph(&self) {
        *self.graph_cache.write() = None;
    }

    // Search cache operations

    pub fn get_search_results(&self, query: &str) -> Option<Vec<Uuid>> {
        self.search_cache.write().get(query).cloned()
    }

    pub fn put_search_results(&self, query: String, results: Vec<Uuid>) {
        self.search_cache.write().put(query, results);
    }

    pub fn clear_search_cache(&self) {
        self.search_cache.write().clear();
    }

    // General operations

    pub fn clear_all(&self) {
        self.clear_notes();
        self.invalidate_graph();
        self.clear_search_cache();
    }

    pub fn get_stats(&self) -> CacheStats {
        CacheStats {
            note_cache_size: self.note_cache.read().len(),
            graph_cached: self.graph_cache.read().is_some(),
            search_cache_size: self.search_cache.read().len(),
        }
    }
}

impl Default for CacheManager {
    fn default() -> Self {
        Self::new(10000) // Cache up to 10k notes
    }
}

#[derive(Debug, Clone)]
pub struct CacheStats {
    pub note_cache_size: usize,
    pub graph_cached: bool,
    pub search_cache_size: usize,
}
