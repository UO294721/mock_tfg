# Architecture Documentation

## System Overview

Knowledge Notes is built using a layered architecture pattern with clear separation of concerns.

## Backend Architecture (Rust)

### Layer 1: Domain Layer
**Location:** `src-tauri/src/domain/`

Pure business logic with no dependencies on infrastructure.

- **Note**: Core note entity with metadata
- **Graph**: Knowledge graph structure (nodes, edges)
- **Link**: Note-to-note relationships
- **Tag**: Hierarchical tagging system

### Layer 2: Storage Layer
**Location:** `src-tauri/src/storage/`

Handles all data persistence and caching.

#### Database
- SQLite with WAL mode for concurrent access
- Connection pooling via parking_lot RwLock
- Transaction support for consistency

#### Schema
- Notes table with FTS5 full-text search
- Graph nodes and edges tables
- Tags with hierarchical relationships
- Automatic triggers for FTS synchronization

#### Repository Pattern
- `NoteRepository`: CRUD operations for notes
- `GraphRepository`: Graph operations
- Abstraction over database details

#### Caching
- LRU cache for hot notes (10k entries)
- Graph cache for expensive operations
- Search result caching

### Layer 3: Graph Engine
**Location:** `src-tauri/src/graph/`

Knowledge graph processing and analysis.

#### Engine
- Built on top of petgraph
- BFS/DFS traversal
- Shortest path algorithms
- Connected components

#### Algorithms
- PageRank for importance ranking
- Path finding between notes
- Similarity calculation
- Hub and orphan detection

#### Analysis
- Graph statistics (density, degree)
- Cluster detection
- Insight generation

### Layer 4: Services Layer
**Location:** `src-tauri/src/services/`

Business logic and orchestration.

- **NoteService**: Note management with cache integration
- **GraphService**: Graph operations and analysis
- **SearchService**: Advanced search with filtering

### Layer 5: API Layer
**Location:** `src-tauri/src/api/`

Tauri command handlers exposing functionality to frontend.

- Type-safe command definitions
- Error handling and conversion
- State management

## Frontend Architecture (TypeScript)

### Component Structure

```
components/
├── ui/              # Reusable UI components
│   ├── Button
│   ├── Input
│   ├── Card
│   └── ...
├── editor/          # Note editor components
├── graph/           # Graph visualization
└── sidebar/         # Navigation
```

### State Management

Using Zustand for global state:

- **NotesStore**: Note list and current note
- **GraphStore**: Graph view state
- **SearchStore**: Search state
- **AppStore**: UI state (sidebar, theme)

### API Client

Type-safe wrappers around Tauri invoke:
- Automatic JSON serialization
- Promise-based API
- Error handling

## Data Flow

### Creating a Note

```
Frontend (React)
  ↓ invoke("create_note")
API Layer (commands.rs)
  ↓ NoteService::create_note
Service Layer
  ↓ NoteRepository::create
Storage Layer
  ↓ SQLite INSERT
Database
  ↑ Return Note
  ↑ Cache::put_note
Service Layer
  ↑ Return to API
API Layer
  ↑ JSON response
Frontend (Update UI)
```

### Search Flow

```
Frontend search input
  ↓ invoke("search_notes")
API Layer
  ↓ SearchService::search
Service Layer
  ├─→ Check cache
  │   └─→ Return if cached
  └─→ Repository::search
      ↓ SQLite FTS5 query
      ↓ Rank results
      ↓ Cache results
      ↑ Return SearchResults
```

### Graph Analysis

```
User requests graph
  ↓ invoke("analyze_graph")
API Layer
  ↓ GraphService::analyze_graph
Service Layer
  ├─→ Check graph cache
  └─→ GraphRepository::get_full_graph
      ↓ Load from database
      ↓ Build in-memory graph
      ↓ GraphEngine::analyze
      ↓ Run algorithms (PageRank, etc.)
      ↑ Return analysis
```

## Performance Optimizations

### Database Level
1. **Indexing Strategy**
   - B-tree indexes on foreign keys
   - FTS5 for full-text search
   - Covering indexes where beneficial

2. **Query Optimization**
   - Prepared statements
   - Batch operations
   - LIMIT for pagination

3. **WAL Mode**
   - Concurrent readers
   - Non-blocking writes
   - Better crash recovery

### Application Level
1. **Caching**
   - LRU cache for notes (O(1) access)
   - Graph caching for expensive operations
   - Search result caching

2. **Lazy Loading**
   - Notes loaded on demand
   - Pagination for large lists
   - Incremental graph loading

3. **Async Operations**
   - Non-blocking I/O with Tokio
   - Parallel processing with Rayon
   - Background tasks

### Frontend Level
1. **React Optimization**
   - Memoization with useMemo
   - Virtual scrolling for long lists
   - Debounced search input

2. **State Management**
   - Zustand for minimal re-renders
   - Selective subscriptions
   - Immutable updates

## Scalability

### Handling 1M+ Notes

1. **Storage**
   - SQLite can handle millions of rows
   - FTS5 remains fast with proper indexing
   - Regular VACUUM for optimization

2. **Memory**
   - LRU cache prevents memory bloat
   - Notes not kept in memory
   - Graph built incrementally

3. **Performance**
   - Sub-second search with FTS5
   - Efficient graph algorithms (O(V + E))
   - Parallel processing where applicable

## Security Considerations

1. **SQL Injection**
   - Parameterized queries only
   - No string concatenation

2. **File System**
   - Database in app data directory
   - Proper file permissions

3. **Input Validation**
   - Type checking with TypeScript
   - Rust's strong typing
   - Validation in service layer

## Testing Strategy

### Unit Tests
- Domain logic tests
- Algorithm tests
- Utility function tests

### Integration Tests
- Repository tests with in-memory DB
- Service layer tests
- API command tests

### Performance Tests
- Load testing with large datasets
- Search performance benchmarks
- Graph algorithm benchmarks

## Deployment

### Desktop Application
- Tauri builds native binaries
- Separate builds for Windows/Mac/Linux
- Auto-updater support (future)

### Distribution
- GitHub releases
- Platform-specific installers
- Code signing (production)

## Future Architecture Improvements

1. **Plugin System**
   - WebAssembly plugins
   - JavaScript API
   - Sandboxed execution

2. **Sync Engine**
   - CRDTs for conflict resolution
   - Differential sync
   - End-to-end encryption

3. **Real-time Collaboration**
   - WebSocket server
   - Operational transforms
   - Presence awareness

4. **Enhanced Search**
   - Semantic search with embeddings
   - Natural language queries
   - Machine learning ranking
