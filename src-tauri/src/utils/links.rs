use uuid::Uuid;
use std::collections::HashMap;

/// Utilities for managing note links and references

/// Extract note IDs from content
pub fn extract_note_references(content: &str) -> Vec<Uuid> {
    let note_url_pattern = regex::Regex::new(r"note://([0-9a-f-]+)").unwrap();

    note_url_pattern
        .captures_iter(content)
        .filter_map(|cap| Uuid::parse_str(&cap[1]).ok())
        .collect()
}

/// Build a backlink map from notes
pub fn build_backlink_map(notes: &[(Uuid, String)]) -> HashMap<Uuid, Vec<Uuid>> {
    let mut backlinks: HashMap<Uuid, Vec<Uuid>> = HashMap::new();

    for (note_id, content) in notes {
        let referenced_notes = extract_note_references(content);

        for referenced_id in referenced_notes {
            backlinks
                .entry(referenced_id)
                .or_default()
                .push(*note_id);
        }
    }

    backlinks
}

/// Check if a note is orphaned (no links to or from it)
pub fn is_orphan(note_id: Uuid, content: &str, backlinks: &HashMap<Uuid, Vec<Uuid>>) -> bool {
    let has_outgoing = !extract_note_references(content).is_empty();
    let has_incoming = backlinks.get(&note_id).map(|v| !v.is_empty()).unwrap_or(false);

    !has_outgoing && !has_incoming
}
