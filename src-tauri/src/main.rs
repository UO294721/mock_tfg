// Prevents additional console window on Windows in release builds
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use knowledge_notes::api::*;
use knowledge_notes::storage::Database;
use std::path::PathBuf;
use tracing_subscriber;

fn main() {
    // Initialize logging
    tracing_subscriber::fmt::init();

    // Get app data directory
    let app_data_dir = tauri::api::path::app_data_dir(&tauri::Config::default())
        .unwrap_or_else(|| PathBuf::from("."));

    std::fs::create_dir_all(&app_data_dir).expect("Failed to create app data directory");

    let db_path = app_data_dir.join("knowledge-notes.db");

    // Initialize database
    let database = Database::new(&db_path).expect("Failed to initialize database");

    // Create application state
    let app_state = AppState::new(database);

    // Build and run Tauri application
    tauri::Builder::default()
        .manage(app_state)
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![
            // Note commands
            commands::create_note,
            commands::get_note,
            commands::update_note,
            commands::delete_note,
            commands::list_notes,
            commands::pin_note,
            commands::unpin_note,
            commands::archive_note,
            commands::restore_note,
            commands::add_tag,
            commands::remove_tag,
            // Search commands
            commands::search_notes,
            commands::advanced_search,
            commands::get_notes_by_tag,
            // Graph commands
            commands::get_graph,
            commands::get_subgraph,
            commands::link_notes,
            commands::get_neighbors,
            commands::suggest_related_notes,
            commands::analyze_graph,
            commands::get_insights,
            commands::find_orphan_notes,
            commands::find_hub_notes,
            // Database commands
            commands::get_database_stats,
            commands::optimize_database,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
