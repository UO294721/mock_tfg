use crate::domain::*;
use uuid::Uuid;

/// Graph analysis and insights generation

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct GraphAnalysis {
    pub node_count: usize,
    pub edge_count: usize,
    pub density: f64,
    pub average_degree: f64,
    pub cluster_count: usize,
    pub orphan_count: usize,
    pub hub_nodes: Vec<Uuid>,
    pub top_ranked_nodes: Vec<(Uuid, f64)>,
}

/// Perform comprehensive graph analysis
pub fn analyze_graph(graph: &KnowledgeGraph) -> GraphAnalysis {
    let node_count = graph.node_count();
    let edge_count = graph.edge_count();

    let density = if node_count > 1 {
        edge_count as f64 / (node_count * (node_count - 1)) as f64
    } else {
        0.0
    };

    let average_degree = if node_count > 0 {
        let total_degree: usize = graph
            .nodes
            .keys()
            .map(|&id| {
                let out = graph.get_neighbors(id).map(|n| n.len()).unwrap_or(0);
                let in_count = graph.get_edges_to(id).len();
                out + in_count
            })
            .sum();
        total_degree as f64 / node_count as f64
    } else {
        0.0
    };

    let orphans = super::algorithms::find_orphan_notes(graph);
    let hubs = super::algorithms::find_hub_nodes(graph, 5);

    let pagerank = super::algorithms::calculate_pagerank(graph, 20, 0.85);
    let mut top_ranked: Vec<(Uuid, f64)> = pagerank.into_iter().collect();
    top_ranked.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
    top_ranked.truncate(10);

    GraphAnalysis {
        node_count,
        edge_count,
        density,
        average_degree,
        cluster_count: 0, // Can be computed with more advanced algorithms
        orphan_count: orphans.len(),
        hub_nodes: hubs.into_iter().map(|(id, _)| id).collect(),
        top_ranked_nodes: top_ranked,
    }
}

/// Generate insights about the knowledge graph
pub fn generate_insights(graph: &KnowledgeGraph) -> Vec<GraphInsight> {
    let mut insights = Vec::new();

    let orphans = super::algorithms::find_orphan_notes(graph);
    if !orphans.is_empty() {
        insights.push(GraphInsight {
            insight_type: InsightType::OrphanNotes,
            title: "Disconnected Notes Found".to_string(),
            description: format!(
                "You have {} notes that are not connected to any other notes. Consider linking them to build your knowledge graph.",
                orphans.len()
            ),
            severity: InsightSeverity::Info,
            affected_nodes: orphans,
        });
    }

    let hubs = super::algorithms::find_hub_nodes(graph, 10);
    if !hubs.is_empty() {
        let hub_ids: Vec<Uuid> = hubs.iter().map(|(id, _)| *id).collect();
        insights.push(GraphInsight {
            insight_type: InsightType::HubNodes,
            title: "Hub Notes Identified".to_string(),
            description: format!(
                "Found {} hub notes with many connections. These are central to your knowledge graph.",
                hubs.len()
            ),
            severity: InsightSeverity::Success,
            affected_nodes: hub_ids,
        });
    }

    let density = if graph.node_count() > 1 {
        graph.edge_count() as f64 / (graph.node_count() * (graph.node_count() - 1)) as f64
    } else {
        0.0
    };

    if density < 0.01 && graph.node_count() > 50 {
        insights.push(GraphInsight {
            insight_type: InsightType::LowDensity,
            title: "Sparse Graph Detected".to_string(),
            description: "Your knowledge graph has relatively few connections. Adding more links between related notes can improve discoverability.".to_string(),
            severity: InsightSeverity::Warning,
            affected_nodes: Vec::new(),
        });
    }

    insights
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct GraphInsight {
    pub insight_type: InsightType,
    pub title: String,
    pub description: String,
    pub severity: InsightSeverity,
    pub affected_nodes: Vec<Uuid>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum InsightType {
    OrphanNotes,
    HubNodes,
    LowDensity,
    HighlyClustered,
    MissingConnections,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum InsightSeverity {
    Info,
    Success,
    Warning,
    Error,
}
