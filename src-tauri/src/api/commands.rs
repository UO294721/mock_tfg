use crate::api::AppState;
use crate::domain::*;
use crate::graph::*;
use crate::services::*;
use crate::storage::DatabaseStats;
use tauri::State;
use uuid::Uuid;

// ============================================================================
// Note Commands
// ============================================================================

#[tauri::command]
pub async fn create_note(
    state: State<'_, AppState>,
    title: String,
    content: String,
    content_type: ContentType,
    tags: Option<Vec<String>>,
) -> Result<Note, String> {
    println!("=== CREATE NOTE CALLED ===");
    println!("Title: {}", title);
    println!("Content length: {}", content.len());
    println!("Content type: {:?}", content_type);
    println!("Tags: {:?}", tags);

    let request = CreateNoteRequest {
        title,
        content,
        content_type,
        tags,
    };

    let result = state
        .note_service
        .read()
        .create_note(request)
        .map_err(|e| e.to_string());

    println!("Result: {:?}", result.as_ref().map(|n| &n.id));
    result
}

#[tauri::command]
pub async fn get_note(state: State<'_, AppState>, id: String) -> Result<Note, String> {
    let uuid = Uuid::parse_str(&id).map_err(|e| e.to_string())?;
    state
        .note_service
        .read()
        .get_note(uuid)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn update_note(
    state: State<'_, AppState>,
    id: String,
    title: Option<String>,
    content: Option<String>,
    tags: Option<Vec<String>>,
    metadata: Option<NoteMetadata>,
) -> Result<Note, String> {
    let uuid = Uuid::parse_str(&id).map_err(|e| e.to_string())?;
    let request = UpdateNoteRequest {
        id: uuid,
        title,
        content,
        tags,
        metadata,
    };
    state
        .note_service
        .read()
        .update_note(request)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn delete_note(state: State<'_, AppState>, id: String) -> Result<(), String> {
    let uuid = Uuid::parse_str(&id).map_err(|e| e.to_string())?;
    state
        .note_service
        .read()
        .delete_note(uuid)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn list_notes(
    state: State<'_, AppState>,
    page: usize,
    page_size: usize,
) -> Result<Vec<Note>, String> {
    state
        .note_service
        .read()
        .list_notes(page, page_size)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn pin_note(state: State<'_, AppState>, id: String) -> Result<(), String> {
    let uuid = Uuid::parse_str(&id).map_err(|e| e.to_string())?;
    state
        .note_service
        .read()
        .pin_note(uuid)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn unpin_note(state: State<'_, AppState>, id: String) -> Result<(), String> {
    let uuid = Uuid::parse_str(&id).map_err(|e| e.to_string())?;
    state
        .note_service
        .read()
        .unpin_note(uuid)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn archive_note(state: State<'_, AppState>, id: String) -> Result<(), String> {
    let uuid = Uuid::parse_str(&id).map_err(|e| e.to_string())?;
    state
        .note_service
        .read()
        .archive_note(uuid)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn restore_note(state: State<'_, AppState>, id: String) -> Result<(), String> {
    let uuid = Uuid::parse_str(&id).map_err(|e| e.to_string())?;
    state
        .note_service
        .read()
        .restore_note(uuid)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn add_tag(
    state: State<'_, AppState>,
    note_id: String,
    tag: String,
) -> Result<(), String> {
    let uuid = Uuid::parse_str(&note_id).map_err(|e| e.to_string())?;
    state
        .note_service
        .read()
        .add_tag(uuid, tag)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn remove_tag(
    state: State<'_, AppState>,
    note_id: String,
    tag: String,
) -> Result<(), String> {
    let uuid = Uuid::parse_str(&note_id).map_err(|e| e.to_string())?;
    state
        .note_service
        .read()
        .remove_tag(uuid, &tag)
        .map_err(|e| e.to_string())
}

// ============================================================================
// Search Commands
// ============================================================================

#[tauri::command]
pub async fn search_notes(
    state: State<'_, AppState>,
    query: String,
    limit: usize,
) -> Result<SearchResults, String> {
    state
        .search_service
        .read()
        .search(&query, limit)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn advanced_search(
    state: State<'_, AppState>,
    options: SearchOptions,
) -> Result<SearchResults, String> {
    state
        .search_service
        .read()
        .advanced_search(options)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_notes_by_tag(
    state: State<'_, AppState>,
    tag: String,
) -> Result<Vec<Note>, String> {
    state
        .note_service
        .read()
        .get_notes_by_tag(&tag)
        .map_err(|e| e.to_string())
}

// ============================================================================
// Graph Commands
// ============================================================================

#[tauri::command]
pub async fn get_graph(state: State<'_, AppState>) -> Result<KnowledgeGraph, String> {
    state
        .graph_service
        .read()
        .get_graph()
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_subgraph(
    state: State<'_, AppState>,
    note_id: String,
    depth: usize,
) -> Result<GraphView, String> {
    let uuid = Uuid::parse_str(&note_id).map_err(|e| e.to_string())?;
    state
        .graph_service
        .read()
        .get_subgraph(uuid, depth)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn link_notes(
    state: State<'_, AppState>,
    source_id: String,
    target_id: String,
    edge_type: EdgeType,
) -> Result<(), String> {
    let source_uuid = Uuid::parse_str(&source_id).map_err(|e| e.to_string())?;
    let target_uuid = Uuid::parse_str(&target_id).map_err(|e| e.to_string())?;
    state
        .graph_service
        .read()
        .link_notes(source_uuid, target_uuid, edge_type)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_neighbors(
    state: State<'_, AppState>,
    note_id: String,
) -> Result<Vec<GraphNode>, String> {
    let uuid = Uuid::parse_str(&note_id).map_err(|e| e.to_string())?;
    state
        .graph_service
        .read()
        .get_note_neighbors(uuid)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn suggest_related_notes(
    state: State<'_, AppState>,
    note_id: String,
    max_suggestions: usize,
) -> Result<Vec<String>, String> {
    let uuid = Uuid::parse_str(&note_id).map_err(|e| e.to_string())?;
    let suggestions = state
        .graph_service
        .read()
        .suggest_related_notes(uuid, max_suggestions)
        .map_err(|e| e.to_string())?;

    Ok(suggestions.into_iter().map(|id| id.to_string()).collect())
}

#[tauri::command]
pub async fn analyze_graph(state: State<'_, AppState>) -> Result<GraphAnalysis, String> {
    state
        .graph_service
        .read()
        .analyze_graph()
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_insights(state: State<'_, AppState>) -> Result<Vec<GraphInsight>, String> {
    state
        .graph_service
        .read()
        .get_insights()
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn find_orphan_notes(state: State<'_, AppState>) -> Result<Vec<String>, String> {
    let orphans = state
        .graph_service
        .read()
        .find_orphan_notes()
        .map_err(|e| e.to_string())?;

    Ok(orphans.into_iter().map(|id| id.to_string()).collect())
}

#[tauri::command]
pub async fn find_hub_notes(
    state: State<'_, AppState>,
    threshold: usize,
) -> Result<Vec<(String, usize)>, String> {
    let hubs = state
        .graph_service
        .read()
        .find_hub_notes(threshold)
        .map_err(|e| e.to_string())?;

    Ok(hubs
        .into_iter()
        .map(|(id, degree)| (id.to_string(), degree))
        .collect())
}

// ============================================================================
// Database Commands
// ============================================================================

#[tauri::command]
pub async fn get_database_stats(state: State<'_, AppState>) -> Result<DatabaseStats, String> {
    state
        .inner()
        .database
        .get_stats()
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn optimize_database(state: State<'_, AppState>) -> Result<(), String> {
    state
        .inner()
        .database
        .optimize()
        .map_err(|e| e.to_string())
}
