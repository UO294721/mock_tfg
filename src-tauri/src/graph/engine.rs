use crate::domain::*;
use petgraph::graph::{DiGraph, NodeIndex};
use petgraph::Direction;
use std::collections::{HashMap, VecDeque};
use uuid::Uuid;

/// High-performance graph engine for knowledge graph operations
pub struct GraphEngine {
    petgraph: DiGraph<Uuid, EdgeType>,
    uuid_to_index: HashMap<Uuid, NodeIndex>,
    index_to_uuid: HashMap<NodeIndex, Uuid>,
}

impl GraphEngine {
    pub fn new() -> Self {
        Self {
            petgraph: DiGraph::new(),
            uuid_to_index: HashMap::new(),
            index_to_uuid: HashMap::new(),
        }
    }

    /// Build the graph engine from a knowledge graph
    pub fn from_knowledge_graph(graph: &KnowledgeGraph) -> Self {
        let mut engine = Self::new();

        // Add all nodes
        for (uuid, _) in &graph.nodes {
            engine.add_node(*uuid);
        }

        // Add all edges
        for edge in graph.edges.values() {
            engine.add_edge(edge.source_id, edge.target_id, edge.edge_type.clone());
        }

        engine
    }

    /// Add a node to the graph
    pub fn add_node(&mut self, uuid: Uuid) -> NodeIndex {
        if let Some(&idx) = self.uuid_to_index.get(&uuid) {
            return idx;
        }

        let idx = self.petgraph.add_node(uuid);
        self.uuid_to_index.insert(uuid, idx);
        self.index_to_uuid.insert(idx, uuid);
        idx
    }

    /// Add an edge between two nodes
    pub fn add_edge(&mut self, source: Uuid, target: Uuid, edge_type: EdgeType) {
        let source_idx = self.add_node(source);
        let target_idx = self.add_node(target);
        self.petgraph.add_edge(source_idx, target_idx, edge_type);
    }

    /// Get all neighbors of a node
    pub fn get_neighbors(&self, node: Uuid) -> Vec<Uuid> {
        if let Some(&idx) = self.uuid_to_index.get(&node) {
            self.petgraph
                .neighbors_directed(idx, Direction::Outgoing)
                .filter_map(|neighbor_idx| self.index_to_uuid.get(&neighbor_idx).copied())
                .collect()
        } else {
            Vec::new()
        }
    }

    /// Get all nodes within a certain depth from a starting node (BFS)
    pub fn get_neighborhood(&self, start: Uuid, max_depth: usize) -> Vec<(Uuid, usize)> {
        if let Some(&start_idx) = self.uuid_to_index.get(&start) {
            let mut visited = HashMap::new();
            let mut queue = VecDeque::new();
            queue.push_back((start_idx, 0));
            visited.insert(start_idx, 0);

            while let Some((current_idx, depth)) = queue.pop_front() {
                if depth >= max_depth {
                    continue;
                }

                for neighbor_idx in self.petgraph.neighbors_directed(current_idx, Direction::Outgoing) {
                    if !visited.contains_key(&neighbor_idx) {
                        visited.insert(neighbor_idx, depth + 1);
                        queue.push_back((neighbor_idx, depth + 1));
                    }
                }
            }

            visited
                .into_iter()
                .filter_map(|(idx, depth)| {
                    self.index_to_uuid.get(&idx).map(|&uuid| (uuid, depth))
                })
                .collect()
        } else {
            Vec::new()
        }
    }

    /// Find shortest path between two nodes
    pub fn find_path(&self, start: Uuid, end: Uuid) -> Option<Vec<Uuid>> {
        let start_idx = self.uuid_to_index.get(&start)?;
        let end_idx = self.uuid_to_index.get(&end)?;

        let path = petgraph::algo::astar(
            &self.petgraph,
            *start_idx,
            |node| node == *end_idx,
            |_| 1,
            |_| 0,
        )?;

        Some(
            path.1
                .into_iter()
                .filter_map(|idx| self.index_to_uuid.get(&idx).copied())
                .collect(),
        )
    }

    /// Get strongly connected components
    pub fn get_clusters(&self) -> Vec<Vec<Uuid>> {
        let sccs = petgraph::algo::kosaraju_scc(&self.petgraph);

        sccs.into_iter()
            .map(|component| {
                component
                    .into_iter()
                    .filter_map(|idx| self.index_to_uuid.get(&idx).copied())
                    .collect()
            })
            .collect()
    }

    /// Calculate node centrality (degree centrality)
    pub fn calculate_centrality(&self) -> HashMap<Uuid, f64> {
        let node_count = self.petgraph.node_count() as f64;
        if node_count <= 1.0 {
            return HashMap::new();
        }

        self.uuid_to_index
            .iter()
            .map(|(&uuid, &idx)| {
                let degree = self.petgraph.neighbors_undirected(idx).count() as f64;
                let centrality = degree / (node_count - 1.0);
                (uuid, centrality)
            })
            .collect()
    }

    /// Find bridge nodes (nodes that connect different clusters)
    pub fn find_bridge_nodes(&self) -> Vec<Uuid> {
        let mut bridges = Vec::new();

        for (&uuid, &idx) in &self.uuid_to_index {
            // A node is a bridge if removing it increases the number of components
            let current_components = petgraph::algo::connected_components(&self.petgraph);

            // Create a temporary graph without this node
            let mut temp_graph = self.petgraph.clone();
            temp_graph.remove_node(idx);

            let new_components = petgraph::algo::connected_components(&temp_graph);

            if new_components > current_components {
                bridges.push(uuid);
            }
        }

        bridges
    }

    /// Get node statistics
    pub fn get_node_stats(&self, node: Uuid) -> Option<NodeStats> {
        let idx = self.uuid_to_index.get(&node)?;

        let in_degree = self.petgraph.neighbors_directed(*idx, Direction::Incoming).count();
        let out_degree = self.petgraph.neighbors_directed(*idx, Direction::Outgoing).count();
        let total_degree = in_degree + out_degree;

        Some(NodeStats {
            node_id: node,
            in_degree,
            out_degree,
            total_degree,
        })
    }

    /// Get graph statistics
    pub fn get_graph_stats(&self) -> GraphStats {
        GraphStats {
            node_count: self.petgraph.node_count(),
            edge_count: self.petgraph.edge_count(),
            density: self.calculate_density(),
            average_degree: self.calculate_average_degree(),
        }
    }

    fn calculate_density(&self) -> f64 {
        let n = self.petgraph.node_count() as f64;
        let e = self.petgraph.edge_count() as f64;

        if n <= 1.0 {
            return 0.0;
        }

        e / (n * (n - 1.0))
    }

    fn calculate_average_degree(&self) -> f64 {
        let n = self.petgraph.node_count() as f64;
        if n == 0.0 {
            return 0.0;
        }

        let total_degree: usize = self
            .petgraph
            .node_indices()
            .map(|idx| self.petgraph.neighbors_undirected(idx).count())
            .sum();

        total_degree as f64 / n
    }
}

impl Default for GraphEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
pub struct NodeStats {
    pub node_id: Uuid,
    pub in_degree: usize,
    pub out_degree: usize,
    pub total_degree: usize,
}

#[derive(Debug, Clone)]
pub struct GraphStats {
    pub node_count: usize,
    pub edge_count: usize,
    pub density: f64,
    pub average_degree: f64,
}
