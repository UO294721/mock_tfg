use super::{ServiceError, ServiceResult};
use crate::domain::*;
use crate::storage::*;
use uuid::Uuid;
use std::collections::HashMap;

/// Service for advanced search operations
pub struct SearchService {
    repository: NoteRepository,
    cache: CacheManager,
}

impl SearchService {
    pub fn new(repository: NoteRepository, cache: CacheManager) -> Self {
        Self { repository, cache }
    }

    /// Full-text search across all notes
    pub fn search(&self, query: &str, limit: usize) -> ServiceResult<SearchResults> {
        // Check cache
        if let Some(cached_ids) = self.cache.get_search_results(query) {
            let results = self.build_search_results(&cached_ids, query)?;
            return Ok(results);
        }

        // Search database
        let notes = self.repository.search(query, limit)?;
        let ids: Vec<Uuid> = notes.iter().map(|n| n.id).collect();

        // Cache results
        self.cache.put_search_results(query.to_string(), ids.clone());

        let results = SearchResults {
            query: query.to_string(),
            results: notes
                .into_iter()
                .map(|note| SearchResult {
                    note_id: note.id,
                    title: note.title,
                    content_preview: self.generate_preview(&note.content, 200),
                    relevance_score: 1.0,
                    matched_tags: note.tags.clone(),
                })
                .collect(),
            total_count: ids.len(),
        };

        Ok(results)
    }

    /// Search with filters
    pub fn advanced_search(&self, options: SearchOptions) -> ServiceResult<SearchResults> {
        let mut all_notes = if let Some(query) = &options.query {
            self.repository.search(query, options.limit.unwrap_or(100))?
        } else {
            self.repository.get_all(options.limit.unwrap_or(100), 0)?
        };

        // Apply filters
        if let Some(tags) = &options.tags {
            all_notes.retain(|note| tags.iter().any(|tag| note.tags.contains(tag)));
        }

        if let Some(date_from) = options.date_from {
            all_notes.retain(|note| note.created_at >= date_from);
        }

        if let Some(date_to) = options.date_to {
            all_notes.retain(|note| note.created_at <= date_to);
        }

        if let Some(content_type) = &options.content_type {
            all_notes.retain(|note| &note.content_type == content_type);
        }

        let total_count = all_notes.len();

        let results = SearchResults {
            query: options.query.unwrap_or_default(),
            results: all_notes
                .into_iter()
                .map(|note| SearchResult {
                    note_id: note.id,
                    title: note.title,
                    content_preview: self.generate_preview(&note.content, 200),
                    relevance_score: 1.0,
                    matched_tags: note.tags.clone(),
                })
                .collect(),
            total_count,
        };

        Ok(results)
    }

    /// Search by tags
    pub fn search_by_tags(&self, tags: Vec<String>) -> ServiceResult<Vec<Note>> {
        let mut results = Vec::new();

        for tag in tags {
            let notes = self.repository.get_by_tag(&tag)?;
            results.extend(notes);
        }

        // Remove duplicates
        results.sort_by_key(|n| n.id);
        results.dedup_by_key(|n| n.id);

        Ok(results)
    }

    /// Get tag suggestions based on partial input
    pub fn suggest_tags(&self, partial: &str, limit: usize) -> ServiceResult<Vec<String>> {
        // This would require a tags table query - simplified for now
        Ok(Vec::new())
    }

    // Helper methods

    fn build_search_results(&self, ids: &[Uuid], query: &str) -> ServiceResult<SearchResults> {
        let notes: Vec<Note> = ids
            .iter()
            .filter_map(|id| {
                if let Some(note) = self.cache.get_note(id) {
                    Some(note)
                } else {
                    self.repository.get(*id).ok()
                }
            })
            .collect();

        let results = SearchResults {
            query: query.to_string(),
            results: notes
                .into_iter()
                .map(|note| SearchResult {
                    note_id: note.id,
                    title: note.title,
                    content_preview: self.generate_preview(&note.content, 200),
                    relevance_score: 1.0,
                    matched_tags: note.tags.clone(),
                })
                .collect(),
            total_count: ids.len(),
        };

        Ok(results)
    }

    fn generate_preview(&self, content: &str, max_length: usize) -> String {
        if content.len() <= max_length {
            content.to_string()
        } else {
            let mut preview = content.chars().take(max_length).collect::<String>();
            preview.push_str("...");
            preview
        }
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SearchResults {
    pub query: String,
    pub results: Vec<SearchResult>,
    pub total_count: usize,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SearchResult {
    pub note_id: Uuid,
    pub title: String,
    pub content_preview: String,
    pub relevance_score: f64,
    pub matched_tags: Vec<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SearchOptions {
    pub query: Option<String>,
    pub tags: Option<Vec<String>>,
    pub content_type: Option<ContentType>,
    pub date_from: Option<chrono::DateTime<chrono::Utc>>,
    pub date_to: Option<chrono::DateTime<chrono::Utc>>,
    pub limit: Option<usize>,
}
