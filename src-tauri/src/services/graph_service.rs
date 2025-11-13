use super::{ServiceError, ServiceResult};
use crate::domain::*;
use crate::graph::*;
use crate::storage::*;
use uuid::Uuid;

/// Service for knowledge graph operations
pub struct GraphService {
    graph_repository: GraphRepository,
    cache: CacheManager,
}

impl GraphService {
    pub fn new(graph_repository: GraphRepository, cache: CacheManager) -> Self {
        Self {
            graph_repository,
            cache,
        }
    }

    /// Get the full knowledge graph
    pub fn get_graph(&self) -> ServiceResult<KnowledgeGraph> {
        // Try cache first
        if let Some(graph) = self.cache.get_graph() {
            return Ok(graph);
        }

        // Load from database
        let graph = self.graph_repository.get_full_graph()?;
        self.cache.put_graph(graph.clone());

        Ok(graph)
    }

    /// Add a note to the graph
    pub fn add_note_to_graph(&self, note: &Note) -> ServiceResult<()> {
        let node = GraphNode {
            id: Uuid::new_v4(),
            note_id: note.id,
            title: note.title.clone(),
            node_type: NodeType::Note,
            position: None,
        };

        self.graph_repository.add_node(&node)?;
        self.cache.invalidate_graph();

        Ok(())
    }

    /// Link two notes in the graph
    pub fn link_notes(
        &self,
        source_note_id: Uuid,
        target_note_id: Uuid,
        edge_type: EdgeType,
    ) -> ServiceResult<()> {
        // Get nodes for these notes
        let graph = self.get_graph()?;

        let source_node = graph
            .nodes
            .values()
            .find(|n| n.note_id == source_note_id)
            .ok_or_else(|| ServiceError::NoteNotFound(source_note_id.to_string()))?;

        let target_node = graph
            .nodes
            .values()
            .find(|n| n.note_id == target_note_id)
            .ok_or_else(|| ServiceError::NoteNotFound(target_note_id.to_string()))?;

        let edge = GraphEdge {
            id: Uuid::new_v4(),
            source_id: source_node.id,
            target_id: target_node.id,
            edge_type,
            weight: 1.0,
            label: None,
        };

        self.graph_repository.add_edge(&edge)?;
        self.cache.invalidate_graph();

        Ok(())
    }

    /// Get neighbors of a note
    pub fn get_note_neighbors(&self, note_id: Uuid) -> ServiceResult<Vec<GraphNode>> {
        let graph = self.get_graph()?;

        let node = graph
            .nodes
            .values()
            .find(|n| n.note_id == note_id)
            .ok_or_else(|| ServiceError::NoteNotFound(note_id.to_string()))?;

        let neighbors = self.graph_repository.get_neighbors(node.id)?;

        Ok(neighbors)
    }

    /// Get a subgraph centered on a note
    pub fn get_subgraph(&self, note_id: Uuid, depth: usize) -> ServiceResult<GraphView> {
        let graph = self.get_graph()?;
        let engine = GraphEngine::from_knowledge_graph(&graph);

        let node = graph
            .nodes
            .values()
            .find(|n| n.note_id == note_id)
            .ok_or_else(|| ServiceError::NoteNotFound(note_id.to_string()))?;

        let neighborhood = engine.get_neighborhood(node.id, depth);
        let node_ids: Vec<Uuid> = neighborhood.iter().map(|(id, _)| *id).collect();

        let nodes: Vec<GraphNode> = node_ids
            .iter()
            .filter_map(|id| graph.nodes.get(id).cloned())
            .collect();

        let edges: Vec<GraphEdge> = graph
            .edges
            .values()
            .filter(|e| node_ids.contains(&e.source_id) && node_ids.contains(&e.target_id))
            .cloned()
            .collect();

        Ok(GraphView {
            nodes,
            edges,
            center_node: Some(node.id),
            depth,
        })
    }

    /// Get suggested related notes
    pub fn suggest_related_notes(
        &self,
        note_id: Uuid,
        max_suggestions: usize,
    ) -> ServiceResult<Vec<Uuid>> {
        let graph = self.get_graph()?;

        let node = graph
            .nodes
            .values()
            .find(|n| n.note_id == note_id)
            .ok_or_else(|| ServiceError::NoteNotFound(note_id.to_string()))?;

        let suggestions = suggest_related_notes(&graph, node.id, max_suggestions);

        // Convert node IDs to note IDs
        let note_suggestions: Vec<Uuid> = suggestions
            .into_iter()
            .filter_map(|(node_id, _score)| {
                graph
                    .nodes
                    .get(&node_id)
                    .map(|node| node.note_id)
            })
            .collect();

        Ok(note_suggestions)
    }

    /// Analyze the graph
    pub fn analyze_graph(&self) -> ServiceResult<GraphAnalysis> {
        let graph = self.get_graph()?;
        let analysis = analyze_graph(&graph);
        Ok(analysis)
    }

    /// Generate graph insights
    pub fn get_insights(&self) -> ServiceResult<Vec<GraphInsight>> {
        let graph = self.get_graph()?;
        let insights = generate_insights(&graph);
        Ok(insights)
    }

    /// Find orphan notes
    pub fn find_orphan_notes(&self) -> ServiceResult<Vec<Uuid>> {
        let graph = self.get_graph()?;
        let orphan_node_ids = find_orphan_notes(&graph);

        // Convert node IDs to note IDs
        let orphan_note_ids: Vec<Uuid> = orphan_node_ids
            .into_iter()
            .filter_map(|node_id| {
                graph
                    .nodes
                    .get(&node_id)
                    .map(|node| node.note_id)
            })
            .collect();

        Ok(orphan_note_ids)
    }

    /// Find hub notes
    pub fn find_hub_notes(&self, threshold: usize) -> ServiceResult<Vec<(Uuid, usize)>> {
        let graph = self.get_graph()?;
        let hub_nodes = find_hub_nodes(&graph, threshold);

        // Convert node IDs to note IDs
        let hub_notes: Vec<(Uuid, usize)> = hub_nodes
            .into_iter()
            .filter_map(|(node_id, degree)| {
                graph
                    .nodes
                    .get(&node_id)
                    .map(|node| (node.note_id, degree))
            })
            .collect();

        Ok(hub_notes)
    }
}
