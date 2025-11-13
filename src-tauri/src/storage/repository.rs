use super::{Database, StorageError, StorageResult};
use crate::domain::*;
use rusqlite::{params, Row};
use uuid::Uuid;
use chrono::{DateTime, Utc};

/// Repository for note operations
pub struct NoteRepository {
    db: Database,
}

impl NoteRepository {
    pub fn new(db: Database) -> Self {
        Self { db }
    }

    /// Create a new note
    pub fn create(&self, note: &Note) -> StorageResult<()> {
        self.db.write(|conn| {
            conn.execute(
                "INSERT INTO notes (id, title, content, content_type, created_at, updated_at,
                 word_count, read_time_minutes, is_pinned, is_archived, color, icon, parent_id)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)",
                params![
                    note.id.to_string(),
                    note.title,
                    note.content,
                    serde_json::to_string(&note.content_type)?,
                    note.created_at.to_rfc3339(),
                    note.updated_at.to_rfc3339(),
                    note.metadata.word_count as i64,
                    note.metadata.read_time_minutes as i64,
                    note.metadata.is_pinned as i32,
                    note.metadata.is_archived as i32,
                    note.metadata.color,
                    note.metadata.icon,
                    note.metadata.parent_id.map(|id| id.to_string()),
                ],
            )?;

            // Insert tags
            for tag in &note.tags {
                self.add_tag_to_note_internal(conn, note.id, tag)?;
            }

            Ok(())
        })
    }

    /// Get a note by ID
    pub fn get(&self, id: Uuid) -> StorageResult<Note> {
        self.db.read(|conn| {
            let note = conn.query_row(
                "SELECT id, title, content, content_type, created_at, updated_at,
                 word_count, read_time_minutes, is_pinned, is_archived, color, icon, parent_id
                 FROM notes WHERE id = ?1",
                params![id.to_string()],
                |row| self.map_note_row(row),
            )??;

            Ok(note)
        })
    }

    /// Update a note
    pub fn update(&self, note: &Note) -> StorageResult<()> {
        self.db.write(|conn| {
            conn.execute(
                "UPDATE notes SET title = ?2, content = ?3, content_type = ?4, updated_at = ?5,
                 word_count = ?6, read_time_minutes = ?7, is_pinned = ?8, is_archived = ?9,
                 color = ?10, icon = ?11, parent_id = ?12
                 WHERE id = ?1",
                params![
                    note.id.to_string(),
                    note.title,
                    note.content,
                    serde_json::to_string(&note.content_type)?,
                    note.updated_at.to_rfc3339(),
                    note.metadata.word_count as i64,
                    note.metadata.read_time_minutes as i64,
                    note.metadata.is_pinned as i32,
                    note.metadata.is_archived as i32,
                    note.metadata.color,
                    note.metadata.icon,
                    note.metadata.parent_id.map(|id| id.to_string()),
                ],
            )?;

            // Update tags (simple approach: delete and re-insert)
            conn.execute(
                "DELETE FROM note_tags WHERE note_id = ?1",
                params![note.id.to_string()],
            )?;

            for tag in &note.tags {
                self.add_tag_to_note_internal(conn, note.id, tag)?;
            }

            Ok(())
        })
    }

    /// Delete a note
    pub fn delete(&self, id: Uuid) -> StorageResult<()> {
        self.db.write(|conn| {
            let affected = conn.execute(
                "DELETE FROM notes WHERE id = ?1",
                params![id.to_string()],
            )?;

            if affected == 0 {
                return Err(StorageError::NoteNotFound(id.to_string()));
            }

            Ok(())
        })
    }

    /// Get all notes (paginated)
    pub fn get_all(&self, limit: usize, offset: usize) -> StorageResult<Vec<Note>> {
        self.db.read(|conn| {
            let mut stmt = conn.prepare(
                "SELECT id, title, content, content_type, created_at, updated_at,
                 word_count, read_time_minutes, is_pinned, is_archived, color, icon, parent_id
                 FROM notes
                 WHERE is_archived = 0
                 ORDER BY is_pinned DESC, updated_at DESC
                 LIMIT ?1 OFFSET ?2",
            )?;

            let notes = stmt
                .query_map(params![limit as i64, offset as i64], |row| {
                    self.map_note_row(row)
                })?
                .collect::<Result<Vec<_>, _>>()?
                .into_iter()
                .collect::<Result<Vec<_>, _>>()?;

            Ok(notes)
        })
    }

    /// Search notes using full-text search
    pub fn search(&self, query: &str, limit: usize) -> StorageResult<Vec<Note>> {
        self.db.read(|conn| {
            let mut stmt = conn.prepare(
                "SELECT n.id, n.title, n.content, n.content_type, n.created_at, n.updated_at,
                 n.word_count, n.read_time_minutes, n.is_pinned, n.is_archived, n.color, n.icon, n.parent_id
                 FROM notes n
                 INNER JOIN notes_fts fts ON n.id = fts.note_id
                 WHERE notes_fts MATCH ?1
                 ORDER BY rank
                 LIMIT ?2",
            )?;

            let notes = stmt
                .query_map(params![query, limit as i64], |row| {
                    self.map_note_row(row)
                })?
                .collect::<Result<Vec<_>, _>>()?
                .into_iter()
                .collect::<Result<Vec<_>, _>>()?;

            Ok(notes)
        })
    }

    /// Get notes by tag
    pub fn get_by_tag(&self, tag_name: &str) -> StorageResult<Vec<Note>> {
        self.db.read(|conn| {
            let mut stmt = conn.prepare(
                "SELECT n.id, n.title, n.content, n.content_type, n.created_at, n.updated_at,
                 n.word_count, n.read_time_minutes, n.is_pinned, n.is_archived, n.color, n.icon, n.parent_id
                 FROM notes n
                 INNER JOIN note_tags nt ON n.id = nt.note_id
                 INNER JOIN tags t ON nt.tag_id = t.id
                 WHERE t.name = ?1 AND n.is_archived = 0
                 ORDER BY n.updated_at DESC",
            )?;

            let notes = stmt
                .query_map(params![tag_name], |row| self.map_note_row(row))?
                .collect::<Result<Vec<_>, _>>()?
                .into_iter()
                .collect::<Result<Vec<_>, _>>()?;

            Ok(notes)
        })
    }

    // Helper methods

    fn map_note_row(&self, row: &Row) -> rusqlite::Result<Result<Note, StorageError>> {
        let id: String = row.get(0)?;
        let title: String = row.get(1)?;
        let content: String = row.get(2)?;
        let content_type_str: String = row.get(3)?;
        let created_at_str: String = row.get(4)?;
        let updated_at_str: String = row.get(5)?;
        let word_count: i64 = row.get(6)?;
        let read_time: i64 = row.get(7)?;
        let is_pinned: i32 = row.get(8)?;
        let is_archived: i32 = row.get(9)?;
        let color: Option<String> = row.get(10)?;
        let icon: Option<String> = row.get(11)?;
        let parent_id_str: Option<String> = row.get(12)?;

        let note_result: Result<Note, StorageError> = (|| {
            let id = Uuid::parse_str(&id)
                .map_err(|e| StorageError::InvalidOperation(format!("Invalid UUID: {}", e)))?;

            let content_type: ContentType = serde_json::from_str(&content_type_str)?;

            let created_at = DateTime::parse_from_rfc3339(&created_at_str)
                .map_err(|e| StorageError::InvalidOperation(format!("Invalid date: {}", e)))?
                .with_timezone(&Utc);

            let updated_at = DateTime::parse_from_rfc3339(&updated_at_str)
                .map_err(|e| StorageError::InvalidOperation(format!("Invalid date: {}", e)))?
                .with_timezone(&Utc);

            let parent_id = parent_id_str
                .as_ref()
                .map(|s| Uuid::parse_str(s))
                .transpose()
                .map_err(|e| StorageError::InvalidOperation(format!("Invalid UUID: {}", e)))?;

            // Get tags for this note
            let tags = self.get_tags_for_note_internal(id)?;

            Ok(Note {
                id,
                title,
                content,
                content_type,
                created_at,
                updated_at,
                tags,
                metadata: NoteMetadata {
                    word_count: word_count as usize,
                    read_time_minutes: read_time as u32,
                    is_pinned: is_pinned != 0,
                    is_archived: is_archived != 0,
                    color,
                    icon,
                    parent_id,
                },
            })
        })();

        Ok(note_result)
    }

    fn get_tags_for_note_internal(&self, note_id: Uuid) -> StorageResult<Vec<String>> {
        self.db.read(|conn| {
            let mut stmt = conn.prepare(
                "SELECT t.name FROM tags t
                 INNER JOIN note_tags nt ON t.id = nt.tag_id
                 WHERE nt.note_id = ?1",
            )?;

            let tags = stmt
                .query_map(params![note_id.to_string()], |row| row.get(0))?
                .collect::<Result<Vec<String>, _>>()?;

            Ok(tags)
        })
    }

    fn add_tag_to_note_internal(
        &self,
        conn: &rusqlite::Connection,
        note_id: Uuid,
        tag_name: &str,
    ) -> StorageResult<()> {
        // Get or create tag
        let tag_id: String = match conn.query_row(
            "SELECT id FROM tags WHERE name = ?1",
            params![tag_name],
            |row| row.get(0),
        ) {
            Ok(id) => id,
            Err(_) => {
                let new_tag = Tag::new(tag_name.to_string(), None);
                conn.execute(
                    "INSERT INTO tags (id, name, color, parent_tag_id, note_count)
                     VALUES (?1, ?2, ?3, ?4, ?5)",
                    params![
                        new_tag.id.to_string(),
                        new_tag.name,
                        new_tag.color,
                        new_tag.parent_tag_id.map(|id| id.to_string()),
                        0
                    ],
                )?;
                new_tag.id.to_string()
            }
        };

        // Link tag to note
        conn.execute(
            "INSERT OR IGNORE INTO note_tags (note_id, tag_id) VALUES (?1, ?2)",
            params![note_id.to_string(), tag_id],
        )?;

        // Update tag count
        conn.execute(
            "UPDATE tags SET note_count = (
                SELECT COUNT(*) FROM note_tags WHERE tag_id = ?1
             ) WHERE id = ?1",
            params![tag_id],
        )?;

        Ok(())
    }
}

/// Repository for graph operations
pub struct GraphRepository {
    db: Database,
}

impl GraphRepository {
    pub fn new(db: Database) -> Self {
        Self { db }
    }

    /// Add a node to the graph
    pub fn add_node(&self, node: &GraphNode) -> StorageResult<()> {
        self.db.write(|conn| {
            conn.execute(
                "INSERT INTO graph_nodes (id, note_id, title, node_type, position_x, position_y)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                params![
                    node.id.to_string(),
                    node.note_id.to_string(),
                    node.title,
                    serde_json::to_string(&node.node_type)?,
                    node.position.as_ref().map(|p| p.x),
                    node.position.as_ref().map(|p| p.y),
                ],
            )?;
            Ok(())
        })
    }

    /// Add an edge to the graph
    pub fn add_edge(&self, edge: &GraphEdge) -> StorageResult<()> {
        self.db.write(|conn| {
            conn.execute(
                "INSERT INTO graph_edges (id, source_id, target_id, edge_type, weight, label)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                params![
                    edge.id.to_string(),
                    edge.source_id.to_string(),
                    edge.target_id.to_string(),
                    serde_json::to_string(&edge.edge_type)?,
                    edge.weight,
                    edge.label,
                ],
            )?;
            Ok(())
        })
    }

    /// Get the full graph
    pub fn get_full_graph(&self) -> StorageResult<KnowledgeGraph> {
        self.db.read(|conn| {
            let mut graph = KnowledgeGraph::new();

            // Load nodes
            let mut stmt = conn.prepare(
                "SELECT id, note_id, title, node_type, position_x, position_y FROM graph_nodes",
            )?;

            let nodes = stmt.query_map([], |row| {
                let id: String = row.get(0)?;
                let note_id: String = row.get(1)?;
                let title: String = row.get(2)?;
                let node_type_str: String = row.get(3)?;
                let position_x: Option<f64> = row.get(4)?;
                let position_y: Option<f64> = row.get(5)?;

                Ok((id, note_id, title, node_type_str, position_x, position_y))
            })?;

            for node_result in nodes {
                let (id, note_id, title, node_type_str, position_x, position_y) = node_result?;

                let node = GraphNode {
                    id: Uuid::parse_str(&id).map_err(|e| {
                        rusqlite::Error::ToSqlConversionFailure(Box::new(e))
                    })?,
                    note_id: Uuid::parse_str(&note_id).map_err(|e| {
                        rusqlite::Error::ToSqlConversionFailure(Box::new(e))
                    })?,
                    title,
                    node_type: serde_json::from_str(&node_type_str).map_err(|e| {
                        rusqlite::Error::ToSqlConversionFailure(Box::new(e))
                    })?,
                    position: position_x.and_then(|x| position_y.map(|y| Position { x, y })),
                };

                graph.add_node(node);
            }

            // Load edges
            let mut stmt = conn.prepare(
                "SELECT id, source_id, target_id, edge_type, weight, label FROM graph_edges",
            )?;

            let edges = stmt.query_map([], |row| {
                let id: String = row.get(0)?;
                let source_id: String = row.get(1)?;
                let target_id: String = row.get(2)?;
                let edge_type_str: String = row.get(3)?;
                let weight: f64 = row.get(4)?;
                let label: Option<String> = row.get(5)?;

                Ok((id, source_id, target_id, edge_type_str, weight, label))
            })?;

            for edge_result in edges {
                let (id, source_id, target_id, edge_type_str, weight, label) = edge_result?;

                let edge = GraphEdge {
                    id: Uuid::parse_str(&id).map_err(|e| {
                        rusqlite::Error::ToSqlConversionFailure(Box::new(e))
                    })?,
                    source_id: Uuid::parse_str(&source_id).map_err(|e| {
                        rusqlite::Error::ToSqlConversionFailure(Box::new(e))
                    })?,
                    target_id: Uuid::parse_str(&target_id).map_err(|e| {
                        rusqlite::Error::ToSqlConversionFailure(Box::new(e))
                    })?,
                    edge_type: serde_json::from_str(&edge_type_str).map_err(|e| {
                        rusqlite::Error::ToSqlConversionFailure(Box::new(e))
                    })?,
                    weight,
                    label,
                };

                graph.add_edge(edge);
            }

            Ok(graph)
        })
    }

    /// Get neighbors of a node
    pub fn get_neighbors(&self, node_id: Uuid) -> StorageResult<Vec<GraphNode>> {
        self.db.read(|conn| {
            let mut stmt = conn.prepare(
                "SELECT DISTINCT n.id, n.note_id, n.title, n.node_type, n.position_x, n.position_y
                 FROM graph_nodes n
                 INNER JOIN graph_edges e ON n.id = e.target_id
                 WHERE e.source_id = ?1",
            )?;

            let nodes = stmt
                .query_map(params![node_id.to_string()], |row| {
                    let id: String = row.get(0)?;
                    let note_id: String = row.get(1)?;
                    let title: String = row.get(2)?;
                    let node_type_str: String = row.get(3)?;
                    let position_x: Option<f64> = row.get(4)?;
                    let position_y: Option<f64> = row.get(5)?;

                    Ok((id, note_id, title, node_type_str, position_x, position_y))
                })?
                .collect::<Result<Vec<_>, _>>()?;

            let result = nodes
                .into_iter()
                .map(|(id, note_id, title, node_type_str, position_x, position_y)| {
                    let node = GraphNode {
                        id: Uuid::parse_str(&id).map_err(|e| {
                            StorageError::InvalidOperation(format!("Invalid UUID: {}", e))
                        })?,
                        note_id: Uuid::parse_str(&note_id).map_err(|e| {
                            StorageError::InvalidOperation(format!("Invalid UUID: {}", e))
                        })?,
                        title,
                        node_type: serde_json::from_str(&node_type_str)?,
                        position: position_x.and_then(|x| position_y.map(|y| Position { x, y })),
                    };
                    Ok(node)
                })
                .collect::<Result<Vec<_>, StorageError>>()?;

            Ok(result)
        })
    }
}
