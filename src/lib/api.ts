import { invoke } from "@tauri-apps/api/core";
import type {
  Note,
  CreateNoteRequest,
  UpdateNoteRequest,
  SearchResults,
  SearchOptions,
  KnowledgeGraph,
  GraphView,
  GraphNode,
  EdgeType,
  GraphAnalysis,
  GraphInsight,
  DatabaseStats,
} from "./types";

// Note API
export const noteApi = {
  create: (request: CreateNoteRequest): Promise<Note> =>
    invoke("create_note", request),

  get: (id: string): Promise<Note> =>
    invoke("get_note", { id }),

  update: (request: UpdateNoteRequest): Promise<Note> =>
    invoke("update_note", request),

  delete: (id: string): Promise<void> =>
    invoke("delete_note", { id }),

  list: (page: number, pageSize: number): Promise<Note[]> =>
    invoke("list_notes", { page, pageSize }),

  pin: (id: string): Promise<void> =>
    invoke("pin_note", { id }),

  unpin: (id: string): Promise<void> =>
    invoke("unpin_note", { id }),

  archive: (id: string): Promise<void> =>
    invoke("archive_note", { id }),

  restore: (id: string): Promise<void> =>
    invoke("restore_note", { id }),

  addTag: (noteId: string, tag: string): Promise<void> =>
    invoke("add_tag", { noteId, tag }),

  removeTag: (noteId: string, tag: string): Promise<void> =>
    invoke("remove_tag", { noteId, tag }),
};

// Search API
export const searchApi = {
  search: (query: string, limit: number): Promise<SearchResults> =>
    invoke("search_notes", { query, limit }),

  advancedSearch: (options: SearchOptions): Promise<SearchResults> =>
    invoke("advanced_search", { options }),

  getByTag: (tag: string): Promise<Note[]> =>
    invoke("get_notes_by_tag", { tag }),
};

// Graph API
export const graphApi = {
  getGraph: (): Promise<KnowledgeGraph> =>
    invoke("get_graph"),

  getSubgraph: (noteId: string, depth: number): Promise<GraphView> =>
    invoke("get_subgraph", { noteId, depth }),

  linkNotes: (sourceId: string, targetId: string, edgeType: EdgeType): Promise<void> =>
    invoke("link_notes", { sourceId, targetId, edgeType }),

  getNeighbors: (noteId: string): Promise<GraphNode[]> =>
    invoke("get_neighbors", { noteId }),

  suggestRelated: (noteId: string, maxSuggestions: number): Promise<string[]> =>
    invoke("suggest_related_notes", { noteId, maxSuggestions }),

  analyze: (): Promise<GraphAnalysis> =>
    invoke("analyze_graph"),

  getInsights: (): Promise<GraphInsight[]> =>
    invoke("get_insights"),

  findOrphans: (): Promise<string[]> =>
    invoke("find_orphan_notes"),

  findHubs: (threshold: number): Promise<Array<[string, number]>> =>
    invoke("find_hub_notes", { threshold }),
};

// Database API
export const databaseApi = {
  getStats: (): Promise<DatabaseStats> =>
    invoke("get_database_stats"),

  optimize: (): Promise<void> =>
    invoke("optimize_database"),
};
