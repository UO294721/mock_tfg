use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};

/// Represents a link between notes with additional metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NoteLink {
    pub id: Uuid,
    pub source_note_id: Uuid,
    pub target_note_id: Uuid,
    pub link_type: LinkType,
    pub created_at: DateTime<Utc>,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum LinkType {
    Bidirectional,
    Unidirectional,
    Embed,
}

impl NoteLink {
    pub fn new(
        source_note_id: Uuid,
        target_note_id: Uuid,
        link_type: LinkType,
        description: Option<String>,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            source_note_id,
            target_note_id,
            link_type,
            created_at: Utc::now(),
            description,
        }
    }
}

/// Represents backlinks to a note
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Backlinks {
    pub note_id: Uuid,
    pub links: Vec<BacklinkInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BacklinkInfo {
    pub source_note_id: Uuid,
    pub source_note_title: String,
    pub link_type: LinkType,
    pub context: String, // Surrounding text where the link appears
}
