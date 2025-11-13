use serde::{Deserialize, Serialize};
use uuid::Uuid;
use std::collections::HashSet;

/// Represents a tag that can be applied to notes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tag {
    pub id: Uuid,
    pub name: String,
    pub color: Option<String>,
    pub parent_tag_id: Option<Uuid>,
    pub note_count: usize,
}

impl Tag {
    pub fn new(name: String, color: Option<String>) -> Self {
        Self {
            id: Uuid::new_v4(),
            name,
            color,
            parent_tag_id: None,
            note_count: 0,
        }
    }
}

/// Tag hierarchy and relationships
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TagHierarchy {
    pub tags: Vec<Tag>,
    pub relationships: Vec<TagRelationship>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TagRelationship {
    pub parent_id: Uuid,
    pub child_id: Uuid,
}

/// Collection of notes associated with a tag
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaggedNotes {
    pub tag: Tag,
    pub note_ids: HashSet<Uuid>,
}
