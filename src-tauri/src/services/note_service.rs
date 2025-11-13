use super::ServiceResult;
use crate::domain::*;
use crate::storage::*;
use uuid::Uuid;

/// Service for note management operations
pub struct NoteService {
    repository: NoteRepository,
    cache: CacheManager,
}

impl NoteService {
    pub fn new(repository: NoteRepository, cache: CacheManager) -> Self {
        Self { repository, cache }
    }

    /// Create a new note
    pub fn create_note(&self, request: CreateNoteRequest) -> ServiceResult<Note> {
        let mut note = Note::new(request.title, request.content, request.content_type);

        if let Some(tags) = request.tags {
            note.tags = tags;
        }

        self.repository.create(&note)?;
        self.cache.put_note(note.clone());

        Ok(note)
    }

    /// Get a note by ID
    pub fn get_note(&self, id: Uuid) -> ServiceResult<Note> {
        // Try cache first
        if let Some(note) = self.cache.get_note(&id) {
            return Ok(note);
        }

        // Load from database
        let note = self.repository.get(id)?;
        self.cache.put_note(note.clone());

        Ok(note)
    }

    /// Update a note
    pub fn update_note(&self, request: UpdateNoteRequest) -> ServiceResult<Note> {
        let mut note = self.repository.get(request.id)?;

        if let Some(title) = request.title {
            note.title = title;
            note.updated_at = chrono::Utc::now();
        }

        if let Some(content) = request.content {
            note.update_content(content);
        }

        if let Some(tags) = request.tags {
            note.tags = tags;
        }

        if let Some(metadata) = request.metadata {
            note.metadata = metadata;
            note.updated_at = chrono::Utc::now();
        }

        self.repository.update(&note)?;
        self.cache.put_note(note.clone());
        self.cache.clear_search_cache(); // Invalidate search cache

        Ok(note)
    }

    /// Delete a note
    pub fn delete_note(&self, id: Uuid) -> ServiceResult<()> {
        self.repository.delete(id)?;
        self.cache.invalidate_note(&id);
        self.cache.clear_search_cache();

        Ok(())
    }

    /// Get all notes (paginated)
    pub fn list_notes(&self, page: usize, page_size: usize) -> ServiceResult<Vec<Note>> {
        let offset = page * page_size;
        let notes = self.repository.get_all(page_size, offset)?;
        Ok(notes)
    }

    /// Search notes
    pub fn search_notes(&self, query: &str, limit: usize) -> ServiceResult<Vec<Note>> {
        // Check cache
        if let Some(cached_ids) = self.cache.get_search_results(query) {
            let notes: Vec<Note> = cached_ids
                .iter()
                .filter_map(|id| self.get_note(*id).ok())
                .take(limit)
                .collect();
            return Ok(notes);
        }

        // Search database
        let notes = self.repository.search(query, limit)?;

        // Cache results
        let ids: Vec<Uuid> = notes.iter().map(|n| n.id).collect();
        self.cache.put_search_results(query.to_string(), ids);

        Ok(notes)
    }

    /// Get notes by tag
    pub fn get_notes_by_tag(&self, tag: &str) -> ServiceResult<Vec<Note>> {
        let notes = self.repository.get_by_tag(tag)?;
        Ok(notes)
    }

    /// Pin a note
    pub fn pin_note(&self, id: Uuid) -> ServiceResult<()> {
        let mut note = self.repository.get(id)?;
        note.metadata.is_pinned = true;
        note.updated_at = chrono::Utc::now();
        self.repository.update(&note)?;
        self.cache.put_note(note);
        Ok(())
    }

    /// Unpin a note
    pub fn unpin_note(&self, id: Uuid) -> ServiceResult<()> {
        let mut note = self.repository.get(id)?;
        note.metadata.is_pinned = false;
        note.updated_at = chrono::Utc::now();
        self.repository.update(&note)?;
        self.cache.put_note(note);
        Ok(())
    }

    /// Archive a note
    pub fn archive_note(&self, id: Uuid) -> ServiceResult<()> {
        let mut note = self.repository.get(id)?;
        note.metadata.is_archived = true;
        note.updated_at = chrono::Utc::now();
        self.repository.update(&note)?;
        self.cache.invalidate_note(&id);
        Ok(())
    }

    /// Restore an archived note
    pub fn restore_note(&self, id: Uuid) -> ServiceResult<()> {
        let mut note = self.repository.get(id)?;
        note.metadata.is_archived = false;
        note.updated_at = chrono::Utc::now();
        self.repository.update(&note)?;
        self.cache.put_note(note);
        Ok(())
    }

    /// Add a tag to a note
    pub fn add_tag(&self, note_id: Uuid, tag: String) -> ServiceResult<()> {
        let mut note = self.repository.get(note_id)?;
        note.add_tag(tag);
        self.repository.update(&note)?;
        self.cache.put_note(note);
        Ok(())
    }

    /// Remove a tag from a note
    pub fn remove_tag(&self, note_id: Uuid, tag: &str) -> ServiceResult<()> {
        let mut note = self.repository.get(note_id)?;
        note.remove_tag(tag);
        self.repository.update(&note)?;
        self.cache.put_note(note);
        Ok(())
    }
}
