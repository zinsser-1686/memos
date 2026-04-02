# Contributors

This project is maintained by the open source community.

## Current Maintainers

- Initial development by Zinsser Liu (@zinsser-1686)

## Project Structure

```
memos/
├── src/
│   ├── lib.rs           # Crate root
│   ├── main.rs          # CLI entrypoint
│   ├── config.rs        # Configuration
│   ├── storage/         # Memvid-backed storage
│   │   ├── vault.rs    # Main vault interface
│   │   ├── frame.rs    # Memory frame schema
│   │   └── relations.rs # SQLite sidecar for graph
│   ├── retrieval/       # Search pipeline
│   │   ├── search.rs   # BM25 + vector + RRF
│   │   ├── intent.rs   # Intent classification
│   │   ├── rerank.rs   # Cross-encoder rerank
│   │   └── graph.rs    # Causal graph expansion
│   ├── cognitive/       # Intelligence layer
│   │   ├── extract.rs  # Decision extraction
│   │   ├── graph.rs    # Causal edge building
│   │   ├── amem.rs     # A-MEM evolution
│   │   ├── feedback.rs # Feedback loop
│   │   └── conflict.rs # Conflict detection
│   ├── lifecycle/       # Memory maintenance
│   │   ├── reflect.rs  # Sleep cycle
│   │   ├── decay.rs    # Recency decay
│   │   └── governance.rs # Capacity limits
│   ├── hooks/           # Session lifecycle hooks
│   │   └── events.rs   # Hook event types
│   └── api/            # Transport interfaces
│       ├── mcp.rs      # MCP stdio server
│       └── http.rs     # HTTP REST API
├── tests/              # Integration tests
├── examples/           # Usage examples
└── docs/              # Documentation
```

## Key Design Decisions

### Why Memvid for storage?

- Single-file format (`.mv2`) — truly portable, no external DB needed
- Append-only frames — crash-safe, immutable history
- Built-in HNSW + BM25 + temporal index
- Ultra-fast: P50 0.025ms, P99 0.075ms

### Why not just use Memvid?

Memvid is a storage engine, not a memory system. It handles:
- ✅ Fast retrieval
- ✅ Vector search
- ✅ Full-text search
- ❌ Decision extraction
- ❌ Causal reasoning
- ❌ Memory evolution
- ❌ Feedback loops

### Why not just use ClawMem?

ClawMem uses SQLite + sqlite-vec for storage, which:
- Requires external SQLite installation
- Has separate vector index files
- Lacks the single-file portability of Memvid

### Memos = Best of both

- Storage: Memvid's `.mv2` file format
- Intelligence: ClawMem's cognitive layer
- Result: Fast + Smart + Portable
