import { useEffect, useState, useRef } from "react";
import { useAppStore, useNotesStore } from "@/lib/store";
import { noteApi, databaseApi } from "@/lib/api";
import { Button } from "@/components/ui/Button";
import { Input } from "@/components/ui/Input";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/Card";
import { formatDate, truncate } from "@/lib/utils";
import {
  FileText,
  Search,
  Plus,
  Menu,
  Network,
  Settings,
  Archive,
  Pin,
  Tag,
} from "lucide-react";

function App() {
  const { sidebarOpen, darkMode, toggleSidebar, toggleGraph, graphVisible } = useAppStore();
  const { notes, currentNote, loading, setNotes, setCurrentNote, setLoading } = useNotesStore();
  const [searchQuery, setSearchQuery] = useState("");
  const [editorContent, setEditorContent] = useState("");
  const saveTimeoutRef = useRef<NodeJS.Timeout | null>(null);

  useEffect(() => {
    // Apply dark mode
    if (darkMode) {
      document.documentElement.classList.add("dark");
    } else {
      document.documentElement.classList.remove("dark");
    }
  }, [darkMode]);

  useEffect(() => {
    // Load initial notes
    loadNotes();
    loadStats();
  }, []);

  // Sync editor content when current note changes
  useEffect(() => {
    if (currentNote) {
      setEditorContent(currentNote.content);
    } else {
      setEditorContent("");
    }
  }, [currentNote?.id]);

  // Cleanup timeout on unmount
  useEffect(() => {
    return () => {
      if (saveTimeoutRef.current) {
        clearTimeout(saveTimeoutRef.current);
      }
    };
  }, []);

  const loadNotes = async () => {
    console.log("Loading notes...");
    try {
      setLoading(true);
      const loadedNotes = await noteApi.list(0, 50);
      console.log("Notes loaded:", loadedNotes.length);
      setNotes(loadedNotes);
    } catch (error) {
      console.error("Failed to load notes:", error);
      // Don't fail silently - show empty state
      setNotes([]);
    } finally {
      setLoading(false);
    }
  };

  const loadStats = async () => {
    console.log("Loading stats...");
    try {
      const stats = await databaseApi.getStats();
      console.log("Database stats:", stats);
    } catch (error) {
      console.error("Failed to load stats:", error);
      // Stats are not critical, continue anyway
    }
  };

  const createNewNote = async () => {
    console.log("Creating new note...");
    try {
      const newNote = await noteApi.create({
        title: "Untitled Note",
        content: "# New Note\n\nStart writing...",
        contentType: "Markdown",
        tags: null,
      } as any);
      console.log("Note created successfully:", newNote);
      setNotes([newNote, ...notes]);
      setCurrentNote(newNote);
    } catch (error) {
      console.error("Failed to create note:", error);
      alert(`Failed to create note: ${error}`);
    }
  };

  const selectNote = async (noteId: string) => {
    try {
      const note = await noteApi.get(noteId);
      setCurrentNote(note);
    } catch (error) {
      console.error("Failed to load note:", error);
    }
  };

  const handleContentChange = (content: string) => {
    setEditorContent(content);

    // Clear existing timeout
    if (saveTimeoutRef.current) {
      clearTimeout(saveTimeoutRef.current);
    }

    // Set new timeout to save after user stops typing
    saveTimeoutRef.current = setTimeout(() => {
      saveNote(content);
    }, 1000); // Save 1 second after user stops typing
  };

  const saveNote = async (content: string) => {
    if (!currentNote) return;

    try {
      const updated = await noteApi.update({
        id: currentNote.id,
        content,
      });
      setCurrentNote(updated);

      // Update in the notes list
      setNotes(notes.map(n => n.id === updated.id ? updated : n));
    } catch (error) {
      console.error("Failed to save note:", error);
    }
  };

  return (
    <div className="h-screen flex overflow-hidden bg-background">
      {/* Sidebar */}
      {sidebarOpen && (
        <div className="w-64 border-r border-border flex flex-col bg-card">
          <div className="p-4 border-b border-border">
            <div className="flex items-center justify-between mb-4">
              <h1 className="text-xl font-bold">Knowledge Notes</h1>
              <Button size="icon" variant="ghost" onClick={toggleSidebar}>
                <Menu className="h-5 w-5" />
              </Button>
            </div>
            <div className="relative">
              <Search className="absolute left-2 top-2.5 h-4 w-4 text-muted-foreground" />
              <Input
                placeholder="Search notes..."
                className="pl-8"
                value={searchQuery}
                onChange={(e) => setSearchQuery(e.target.value)}
              />
            </div>
          </div>

          <div className="flex-1 overflow-y-auto p-2">
            <div className="space-y-1">
              {notes.map((note) => (
                <button
                  key={note.id}
                  onClick={() => selectNote(note.id)}
                  className={`w-full text-left p-3 rounded-lg hover:bg-accent transition-colors ${
                    currentNote?.id === note.id ? "bg-accent" : ""
                  }`}
                >
                  <div className="flex items-center gap-2 mb-1">
                    <FileText className="h-4 w-4 text-muted-foreground" />
                    <span className="font-medium text-sm truncate">
                      {note.title}
                    </span>
                    {note.metadata.is_pinned && (
                      <Pin className="h-3 w-3 text-primary ml-auto" />
                    )}
                  </div>
                  <p className="text-xs text-muted-foreground truncate">
                    {truncate(note.content, 60)}
                  </p>
                  <p className="text-xs text-muted-foreground mt-1">
                    {formatDate(note.updated_at)}
                  </p>
                </button>
              ))}
            </div>
          </div>

          <div className="p-2 border-t border-border">
            <Button onClick={createNewNote} className="w-full" size="sm">
              <Plus className="h-4 w-4 mr-2" />
              New Note
            </Button>
          </div>
        </div>
      )}

      {/* Main Content */}
      <div className="flex-1 flex flex-col">
        {/* Header */}
        <div className="h-14 border-b border-border flex items-center justify-between px-4 bg-card">
          <div className="flex items-center gap-2">
            {!sidebarOpen && (
              <Button size="icon" variant="ghost" onClick={toggleSidebar}>
                <Menu className="h-5 w-5" />
              </Button>
            )}
            {currentNote && (
              <span className="font-medium">{currentNote.title}</span>
            )}
          </div>

          <div className="flex items-center gap-2">
            <Button size="icon" variant="ghost" onClick={toggleGraph}>
              <Network className="h-5 w-5" />
            </Button>
            <Button size="icon" variant="ghost">
              <Settings className="h-5 w-5" />
            </Button>
          </div>
        </div>

        {/* Editor */}
        <div className="flex-1 overflow-y-auto p-6">
          {currentNote ? (
            <div className="max-w-4xl mx-auto">
              <textarea
                value={editorContent}
                onChange={(e) => handleContentChange(e.target.value)}
                className="w-full h-full min-h-[600px] bg-transparent border-none outline-none resize-none font-mono text-sm"
                placeholder="Start writing..."
              />
            </div>
          ) : (
            <div className="h-full flex items-center justify-center">
              <Card className="w-96">
                <CardHeader>
                  <CardTitle>Welcome to Knowledge Notes</CardTitle>
                  <CardDescription>
                    Create your first note to get started
                  </CardDescription>
                </CardHeader>
                <CardContent>
                  <Button onClick={createNewNote} className="w-full">
                    <Plus className="h-4 w-4 mr-2" />
                    Create Note
                  </Button>
                </CardContent>
              </Card>
            </div>
          )}
        </div>
      </div>

      {/* Graph View */}
      {graphVisible && (
        <div className="w-96 border-l border-border bg-card p-4">
          <h2 className="text-lg font-semibold mb-4">Knowledge Graph</h2>
          <p className="text-sm text-muted-foreground">
            Graph visualization will appear here
          </p>
        </div>
      )}
    </div>
  );
}

export default App;
