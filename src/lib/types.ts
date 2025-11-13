// Domain types matching Rust backend

export interface Note {
  id: string;
  title: string;
  content: string;
  content_type: ContentType;
  created_at: string;
  updated_at: string;
  tags: string[];
  metadata: NoteMetadata;
}

export type ContentType = "Markdown" | "PlainText" | "RichText";

export interface NoteMetadata {
  word_count: number;
  read_time_minutes: number;
  is_pinned: boolean;
  is_archived: boolean;
  color?: string;
  icon?: string;
  parent_id?: string;
}

export interface CreateNoteRequest {
  title: string;
  content: string;
  content_type: ContentType;
  tags?: string[];
}

export interface UpdateNoteRequest {
  id: string;
  title?: string;
  content?: string;
  tags?: string[];
  metadata?: NoteMetadata;
}

export interface GraphNode {
  id: string;
  note_id: string;
  title: string;
  node_type: NodeType;
  position?: Position;
}

export type NodeType = "Note" | "Concept" | "Tag" | "Reference";

export interface Position {
  x: number;
  y: number;
}

export interface GraphEdge {
  id: string;
  source_id: string;
  target_id: string;
  edge_type: EdgeType;
  weight: number;
  label?: string;
}

export type EdgeType =
  | "Reference"
  | "Related"
  | "Parent"
  | "Child"
  | "Similarity"
  | "Sequence"
  | { Custom: string };

export interface KnowledgeGraph {
  nodes: Record<string, GraphNode>;
  edges: Record<string, GraphEdge>;
  adjacency_list: Record<string, string[]>;
}

export interface GraphView {
  nodes: GraphNode[];
  edges: GraphEdge[];
  center_node?: string;
  depth: number;
}

export interface SearchResult {
  note_id: string;
  title: string;
  content_preview: string;
  relevance_score: number;
  matched_tags: string[];
}

export interface SearchResults {
  query: string;
  results: SearchResult[];
  total_count: number;
}

export interface SearchOptions {
  query?: string;
  tags?: string[];
  content_type?: ContentType;
  date_from?: string;
  date_to?: string;
  limit?: number;
}

export interface GraphAnalysis {
  node_count: number;
  edge_count: number;
  density: number;
  average_degree: number;
  cluster_count: number;
  orphan_count: number;
  hub_nodes: string[];
  top_ranked_nodes: Array<[string, number]>;
}

export interface GraphInsight {
  insight_type: InsightType;
  title: string;
  description: string;
  severity: InsightSeverity;
  affected_nodes: string[];
}

export type InsightType =
  | "OrphanNotes"
  | "HubNodes"
  | "LowDensity"
  | "HighlyClustered"
  | "MissingConnections";

export type InsightSeverity = "Info" | "Success" | "Warning" | "Error";

export interface DatabaseStats {
  note_count: number;
  archived_count: number;
  tag_count: number;
  link_count: number;
  node_count: number;
  edge_count: number;
}
