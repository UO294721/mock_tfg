use crate::domain::*;
use std::collections::{HashMap, HashSet, VecDeque};
use uuid::Uuid;

/// Advanced graph algorithms for knowledge graph analysis

/// Find all paths between two nodes (up to max_depth)
pub fn find_all_paths(
    graph: &KnowledgeGraph,
    start: Uuid,
    end: Uuid,
    max_depth: usize,
) -> Vec<Vec<Uuid>> {
    let mut paths = Vec::new();
    let mut current_path = vec![start];
    let mut visited = HashSet::new();
    visited.insert(start);

    dfs_paths(
        graph,
        start,
        end,
        &mut current_path,
        &mut visited,
        &mut paths,
        max_depth,
    );

    paths
}

fn dfs_paths(
    graph: &KnowledgeGraph,
    current: Uuid,
    end: Uuid,
    current_path: &mut Vec<Uuid>,
    visited: &mut HashSet<Uuid>,
    paths: &mut Vec<Vec<Uuid>>,
    max_depth: usize,
) {
    if current_path.len() > max_depth {
        return;
    }

    if current == end {
        paths.push(current_path.clone());
        return;
    }

    if let Some(neighbors) = graph.get_neighbors(current) {
        for &neighbor in neighbors {
            if !visited.contains(&neighbor) {
                visited.insert(neighbor);
                current_path.push(neighbor);

                dfs_paths(graph, neighbor, end, current_path, visited, paths, max_depth);

                current_path.pop();
                visited.remove(&neighbor);
            }
        }
    }
}

/// Calculate PageRank for nodes in the graph
pub fn calculate_pagerank(
    graph: &KnowledgeGraph,
    iterations: usize,
    damping: f64,
) -> HashMap<Uuid, f64> {
    let node_count = graph.node_count();
    if node_count == 0 {
        return HashMap::new();
    }

    let initial_value = 1.0 / node_count as f64;
    let mut ranks: HashMap<Uuid, f64> = graph
        .nodes
        .keys()
        .map(|&id| (id, initial_value))
        .collect();

    for _ in 0..iterations {
        let mut new_ranks = HashMap::new();

        for &node_id in graph.nodes.keys() {
            let incoming_edges = graph.get_edges_to(node_id);
            let mut rank_sum = 0.0;

            for edge in incoming_edges {
                let source_rank = ranks.get(&edge.source_id).unwrap_or(&0.0);
                let out_degree = graph
                    .get_edges_from(edge.source_id)
                    .len()
                    .max(1) as f64;
                rank_sum += source_rank / out_degree;
            }

            let new_rank = (1.0 - damping) / node_count as f64 + damping * rank_sum;
            new_ranks.insert(node_id, new_rank);
        }

        ranks = new_ranks;
    }

    ranks
}

/// Suggest related notes based on graph proximity and similarity
pub fn suggest_related_notes(
    graph: &KnowledgeGraph,
    note_id: Uuid,
    max_suggestions: usize,
) -> Vec<(Uuid, f64)> {
    let mut scores: HashMap<Uuid, f64> = HashMap::new();

    // Get immediate neighbors
    if let Some(neighbors) = graph.get_neighbors(note_id) {
        for &neighbor in neighbors {
            *scores.entry(neighbor).or_insert(0.0) += 1.0;

            // Get second-degree neighbors
            if let Some(second_neighbors) = graph.get_neighbors(neighbor) {
                for &second_neighbor in second_neighbors {
                    if second_neighbor != note_id {
                        *scores.entry(second_neighbor).or_insert(0.0) += 0.5;
                    }
                }
            }
        }
    }

    // Get reverse connections (backlinks)
    let incoming_edges = graph.get_edges_to(note_id);
    for edge in incoming_edges {
        *scores.entry(edge.source_id).or_insert(0.0) += 0.8;
    }

    // Sort by score and return top suggestions
    let mut suggestions: Vec<(Uuid, f64)> = scores.into_iter().collect();
    suggestions.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
    suggestions.truncate(max_suggestions);

    suggestions
}

/// Find orphan notes (notes with no connections)
pub fn find_orphan_notes(graph: &KnowledgeGraph) -> Vec<Uuid> {
    graph
        .nodes
        .keys()
        .filter(|&&node_id| {
            let has_outgoing = graph
                .get_neighbors(node_id)
                .map(|n| !n.is_empty())
                .unwrap_or(false);
            let has_incoming = !graph.get_edges_to(node_id).is_empty();

            !has_outgoing && !has_incoming
        })
        .copied()
        .collect()
}

/// Find hub nodes (nodes with many connections)
pub fn find_hub_nodes(graph: &KnowledgeGraph, threshold: usize) -> Vec<(Uuid, usize)> {
    let mut hubs: Vec<(Uuid, usize)> = graph
        .nodes
        .keys()
        .filter_map(|&node_id| {
            let out_degree = graph
                .get_neighbors(node_id)
                .map(|n| n.len())
                .unwrap_or(0);
            let in_degree = graph.get_edges_to(node_id).len();
            let total_degree = out_degree + in_degree;

            if total_degree >= threshold {
                Some((node_id, total_degree))
            } else {
                None
            }
        })
        .collect();

    hubs.sort_by(|a, b| b.1.cmp(&a.1));
    hubs
}

/// Calculate similarity between two notes based on shared connections
pub fn calculate_note_similarity(
    graph: &KnowledgeGraph,
    note1: Uuid,
    note2: Uuid,
) -> f64 {
    let neighbors1: HashSet<Uuid> = graph
        .get_neighbors(note1)
        .map(|n| n.iter().copied().collect())
        .unwrap_or_default();

    let neighbors2: HashSet<Uuid> = graph
        .get_neighbors(note2)
        .map(|n| n.iter().copied().collect())
        .unwrap_or_default();

    if neighbors1.is_empty() && neighbors2.is_empty() {
        return 0.0;
    }

    let intersection = neighbors1.intersection(&neighbors2).count();
    let union = neighbors1.union(&neighbors2).count();

    intersection as f64 / union as f64
}
