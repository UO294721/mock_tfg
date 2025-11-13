use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Represents a note in the knowledge graph system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Note {
    pub id: Uuid,
    pub title: String,
    pub content: String,
    pub content_type: ContentType,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub tags: Vec<String>,
    pub metadata: NoteMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ContentType {
    Markdown,
    PlainText,
    RichText,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NoteMetadata {
    pub word_count: usize,
    pub read_time_minutes: u32,
    pub is_pinned: bool,
    pub is_archived: bool,
    pub color: Option<String>,
    pub icon: Option<String>,
    pub parent_id: Option<Uuid>,
}

impl Note {
    pub fn new(title: String, content: String, content_type: ContentType) -> Self {
        let now = Utc::now();
        let word_count = content.split_whitespace().count();
        let read_time = (word_count / 200).max(1) as u32; // Average reading speed

        Self {
            id: Uuid::new_v4(),
            title,
            content,
            content_type,
            created_at: now,
            updated_at: now,
            tags: Vec::new(),
            metadata: NoteMetadata {
                word_count,
                read_time_minutes: read_time,
                is_pinned: false,
                is_archived: false,
                color: None,
                icon: None,
                parent_id: None,
            },
        }
    }

    pub fn update_content(&mut self, content: String) {
        self.content = content;
        self.updated_at = Utc::now();
        self.metadata.word_count = self.content.split_whitespace().count();
        self.metadata.read_time_minutes = (self.metadata.word_count / 200).max(1) as u32;
    }

    pub fn add_tag(&mut self, tag: String) {
        if !self.tags.contains(&tag) {
            self.tags.push(tag);
            self.updated_at = Utc::now();
        }
    }

    pub fn remove_tag(&mut self, tag: &str) {
        self.tags.retain(|t| t != tag);
        self.updated_at = Utc::now();
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateNoteRequest {
    pub title: String,
    pub content: String,
    pub content_type: ContentType,
    pub tags: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateNoteRequest {
    pub id: Uuid,
    pub title: Option<String>,
    pub content: Option<String>,
    pub tags: Option<Vec<String>>,
    pub metadata: Option<NoteMetadata>,
}
