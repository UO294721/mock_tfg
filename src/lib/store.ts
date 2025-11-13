import { create } from "zustand";
import type { Note, GraphView, DatabaseStats } from "./types";

interface NotesState {
  notes: Note[];
  currentNote: Note | null;
  loading: boolean;
  setNotes: (notes: Note[]) => void;
  setCurrentNote: (note: Note | null) => void;
  addNote: (note: Note) => void;
  updateNote: (note: Note) => void;
  deleteNote: (id: string) => void;
  setLoading: (loading: boolean) => void;
}

export const useNotesStore = create<NotesState>((set) => ({
  notes: [],
  currentNote: null,
  loading: false,
  setNotes: (notes) => set({ notes }),
  setCurrentNote: (note) => set({ currentNote: note }),
  addNote: (note) => set((state) => ({ notes: [...state.notes, note] })),
  updateNote: (note) =>
    set((state) => ({
      notes: state.notes.map((n) => (n.id === note.id ? note : n)),
      currentNote: state.currentNote?.id === note.id ? note : state.currentNote,
    })),
  deleteNote: (id) =>
    set((state) => ({
      notes: state.notes.filter((n) => n.id !== id),
      currentNote: state.currentNote?.id === id ? null : state.currentNote,
    })),
  setLoading: (loading) => set({ loading }),
}));

interface GraphState {
  graphView: GraphView | null;
  selectedNode: string | null;
  setGraphView: (view: GraphView | null) => void;
  setSelectedNode: (nodeId: string | null) => void;
}

export const useGraphStore = create<GraphState>((set) => ({
  graphView: null,
  selectedNode: null,
  setGraphView: (view) => set({ graphView: view }),
  setSelectedNode: (nodeId) => set({ selectedNode: nodeId }),
}));

interface SearchState {
  query: string;
  results: any[];
  setQuery: (query: string) => void;
  setResults: (results: any[]) => void;
}

export const useSearchStore = create<SearchState>((set) => ({
  query: "",
  results: [],
  setQuery: (query) => set({ query }),
  setResults: (results) => set({ results }),
}));

interface AppState {
  sidebarOpen: boolean;
  graphVisible: boolean;
  darkMode: boolean;
  stats: DatabaseStats | null;
  toggleSidebar: () => void;
  toggleGraph: () => void;
  toggleDarkMode: () => void;
  setStats: (stats: DatabaseStats | null) => void;
}

export const useAppStore = create<AppState>((set) => ({
  sidebarOpen: true,
  graphVisible: false,
  darkMode: false,
  stats: null,
  toggleSidebar: () => set((state) => ({ sidebarOpen: !state.sidebarOpen })),
  toggleGraph: () => set((state) => ({ graphVisible: !state.graphVisible })),
  toggleDarkMode: () => set((state) => ({ darkMode: !state.darkMode })),
  setStats: (stats) => set({ stats }),
}));
