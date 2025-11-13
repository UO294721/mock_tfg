use serde::{Deserialize, Serialize};
use uuid::Uuid;
use std::collections::{HashMap, HashSet};

/// Represents a node in the knowledge graph
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphNode {
    pub id: Uuid,
    pub note_id: Uuid,
    pub title: String,
    pub node_type: NodeType,
    pub position: Option<Position>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum NodeType {
    Note,
    Concept,
    Tag,
    Reference,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Position {
    pub x: f64,
    pub y: f64,
}

/// Represents an edge connecting two nodes in the knowledge graph
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphEdge {
    pub id: Uuid,
    pub source_id: Uuid,
    pub target_id: Uuid,
    pub edge_type: EdgeType,
    pub weight: f64,
    pub label: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum EdgeType {
    Reference,      // One note references another
    Related,        // Notes are related
    Parent,         // Hierarchical parent-child
    Child,          // Hierarchical child-parent
    Similarity,     // Notes are similar
    Sequence,       // Sequential relationship
    Custom(String), // User-defined relationship
}

/// The complete knowledge graph structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnowledgeGraph {
    pub nodes: HashMap<Uuid, GraphNode>,
    pub edges: HashMap<Uuid, GraphEdge>,
    pub adjacency_list: HashMap<Uuid, HashSet<Uuid>>,
}

impl KnowledgeGraph {
    pub fn new() -> Self {
        Self {
            nodes: HashMap::new(),
            edges: HashMap::new(),
            adjacency_list: HashMap::new(),
        }
    }

    pub fn add_node(&mut self, node: GraphNode) {
        let id = node.id;
        self.nodes.insert(id, node);
        self.adjacency_list.entry(id).or_insert_with(HashSet::new);
    }

    pub fn add_edge(&mut self, edge: GraphEdge) {
        let source = edge.source_id;
        let target = edge.target_id;
        let edge_id = edge.id;

        self.edges.insert(edge_id, edge);
        self.adjacency_list
            .entry(source)
            .or_insert_with(HashSet::new)
            .insert(target);
    }

    pub fn remove_node(&mut self, node_id: Uuid) {
        self.nodes.remove(&node_id);
        self.adjacency_list.remove(&node_id);

        // Remove all edges connected to this node
        let edges_to_remove: Vec<Uuid> = self
            .edges
            .iter()
            .filter(|(_, e)| e.source_id == node_id || e.target_id == node_id)
            .map(|(id, _)| *id)
            .collect();

        for edge_id in edges_to_remove {
            self.edges.remove(&edge_id);
        }

        // Clean up adjacency list
        for neighbors in self.adjacency_list.values_mut() {
            neighbors.remove(&node_id);
        }
    }

    pub fn get_neighbors(&self, node_id: Uuid) -> Option<&HashSet<Uuid>> {
        self.adjacency_list.get(&node_id)
    }

    pub fn get_node(&self, node_id: Uuid) -> Option<&GraphNode> {
        self.nodes.get(&node_id)
    }

    pub fn get_edges_from(&self, node_id: Uuid) -> Vec<&GraphEdge> {
        self.edges
            .values()
            .filter(|e| e.source_id == node_id)
            .collect()
    }

    pub fn get_edges_to(&self, node_id: Uuid) -> Vec<&GraphEdge> {
        self.edges
            .values()
            .filter(|e| e.target_id == node_id)
            .collect()
    }

    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }

    pub fn edge_count(&self) -> usize {
        self.edges.len()
    }
}

impl Default for KnowledgeGraph {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphView {
    pub nodes: Vec<GraphNode>,
    pub edges: Vec<GraphEdge>,
    pub center_node: Option<Uuid>,
    pub depth: usize,
}
