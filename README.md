# Memos — AI Agent Memory System

> **High-performance storage meets autonomous memory evolution.**

Memos combines Memvid's ultra-fast single-file vector storage layer with ClawMem's cognitive engine — autonomous decision extraction, causal graph traversal, A-MEM self-evolution, and feedback-driven decay. Built in Rust for speed, wrapped in TypeScript for developer ergonomics.

## Status

**Planning/Architecture phase** — Specification is frozen, implementation scaffold is being built.

## What This Project Is

Memos is **NOT** a fork of either project. It is a **new architecture** that:

1. Takes Memvid's `.mv2` file format and HNSW/BM25 retrieval engine as the **storage and retrieval backend**
2. Adopts ClawMem's cognitive layer concepts — decision extraction, causal graphs, A-MEM evolution, content-type half-lives — as the **memory intelligence layer**
3. Adds original research where both designs have gaps

## Core Design Principles

| Principle | Rationale |
|-----------|-----------|
| **Storage is dumb, cognition is smart** | Memvid handles bits; Memos handles meaning |
| **Append-only is sacred** | Memories are never deleted, only decayed/archived |
| **Agent-native, not RAG-native** | Memory is for agents, not document retrieval pipelines |
| **Zero external services by default** | Single binary, single file, works offline |
| **Provably correct** | Hybrid search + causal graph + feedback signals |

## Architecture Overview

```
┌─────────────────────────────────────────────────────────────┐
│                        Memos Core                            │
│                                                              │
│  ┌──────────────┐   ┌──────────────┐   ┌──────────────┐    │
│  │  Cognitive   │   │  Retrieval   │   │   Storage    │    │
│  │    Layer    │◄──│    Layer     │◄──│    Layer     │    │
│  │              │   │              │   │              │    │
│  │ · Decision   │   │ · Hybrid    │   │ · Memvid    │    │
│  │   Extraction │   │   Search    │   │   (.mv2)    │    │
│  │ · Causal     │   │ · RRF      │   │ · HNSW      │    │
│  │   Graph      │   │   Fusion    │   │ · BM25       │    │
│  │ · A-MEM      │   │ · Cross-    │   │ · Temporal   │    │
│  │   Evolution  │   │   Encoder   │   │   Index      │    │
│  │ · Feedback   │   │   Rerank   │   │              │    │
│  │   Loop      │   │              │   │              │    │
│  │ · Conflict   │   │              │   │              │    │
│  │   Detection  │   │              │   │              │    │
│  └──────────────┘   └──────────────┘   └──────────────┘    │
│                                                              │
│  ┌──────────────────────────────────────────────────────┐   │
│  │                    Hooks / Events                     │   │
│  │  SessionStart │ UserPrompt │ Stop │ PreCompact       │   │
│  └──────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────┘
```

## Feature Matrix

| Feature | Memvid | ClawMem | Memos |
|---------|--------|---------|-------|
| Single-file storage | ✅ | ❌ | ✅ |
| HNSW vector search | ✅ | ❌ | ✅ |
| BM25 full-text search | ✅ | ❌ | ✅ |
| Temporal index | ✅ | ❌ | ✅ |
| Causal graph | ❌ | ✅ | ✅ |
| A-MEM self-evolution | ❌ | ✅ | ✅ |
| Decision extraction | ❌ | ✅ | ✅ |
| Conflict detection | ❌ | ✅ | ✅ |
| Feedback loop (decay/boost) | ❌ | ✅ | ✅ |
| Content-type half-lives | ❌ | ✅ | ✅ |
| Cross-session handoff | ❌ | ✅ | ✅ |
| MMR diversity reranking | ❌ | ✅ | ✅ |
| Multi-hop reasoning | ❌ | ✅ | ✅ |
| Fragment-level embedding | ❌ | ✅ | ✅ |

## Storage Layer (Memvid)

- **File format**: `.mv2` — single file containing WAL, data segments, FTS index, HNSW index, temporal index, TOC
- **No external DB**: No PostgreSQL, no SQLite, no Redis, no Qdrant
- **Append-only**: New memories are added as immutable frames; old ones are marked decayed/archived, never overwritten
- **Crash-safe**: Committed WAL frames survive abrupt shutdown

## Retrieval Layer (Hybrid)

```
User Query
    │
    ▼
BM25 Probe ──────────────────────────────┐
    │                                     │
    ▼                                     │
Vector Search (HNSW)                     │
    │                                     │
    ▼                                     ▼
Intent Classification ──────────────────►│
(WHY/WHEN/ENTITY/WHAT)                   │
    │                                     │
    ▼                                     │
Reciprocal Rank Fusion ──────────────────►│
    │                                     │
    ▼                                     │
Cross-Encoder Reranking (compact)        │
    │                                     │
    ▼                                     │
Causal Graph Expansion (WHY/ENTITY)       │
    │                                     │
    ▼                                     │
Composite Scoring (A-MEM signals) ──────►│
    │                                     │
    ▼                                     │
MMR Diversity Filter                      │
    │                                     │
    ▼                                     ▼
Ranked Results
```

## Cognitive Layer

### Decision Extraction

On session stop, a local LLM observes the transcript and extracts:
- `type`: decision / observation / preference / fact
- `title`: short description
- `facts`: structured key-value pairs
- `narrative`: free-text summary
- `contradicts`: references to prior decisions that this supersedes

### Causal Graph

- Nodes: memory fragments
- Edges: cause → effect (extracted from decision observations)
- Edge weight: confidence score (0.0–1.0)
- Traversal: MPFP (Multi-Path Fact Propagation) with Forward Push

### A-MEM (Adaptive Memory Evolution)

On each index operation:
1. Extract keywords (3–7 specific terms)
2. Extract tags (3–5 broad categories)
3. Generate context description (1–2 sentences)
4. Find related memories → create semantic links
5. Update neighbor metadata if links changed

### Feedback Loop

- **Surfaced + Referenced** → confidence boost
- **Surfaced + Ignored** → decay signal
- **Referenced** → access_count++
- **Never surfaced, never referenced** → pure recency decay

### Content-Type Half-Lives

| Type | Baseline | Half-life | Decay rule |
|------|----------|-----------|------------|
| `decision` | 0.85 | ∞ | Never decays |
| `hub` | 0.80 | ∞ | Never decays |
| `antipattern` | 0.75 | ∞ | Never decays |
| `research` | 0.70 | 90 days | Linear decay |
| `project` | 0.65 | 120 days | Linear decay |
| `handoff` | 0.60 | 30 days | Fast decay |
| `progress` | 0.50 | 45 days | Linear decay |
| `note` | 0.50 | 60 days | Linear decay |

## Memory Types

```
memory/
├── facts/            # Extracted facts (key-value)
├── observations/     # Decision + narrative observations
├── handoffs/         # Session summaries
├── entities/         # Named entities (people, projects, tools)
├── preferences/      # User preferences
├── antipatterns/     # Negative patterns to avoid
└── resources/        # Static reference (∞ half-life)
```

## Integration Modes

| Mode | Description |
|------|-------------|
| **Rust SDK** | `cargo add memos` — embed in any Rust binary |
| **CLI** | `memos <command>` — shell-friendly |
| **MCP stdio** | Any MCP-compatible client (Claude Code, OpenClaw, etc.) |
| **HTTP API** | REST API for web dashboards or non-MCP agents |

## CLI Reference

```bash
memos init                    # Create vault
memos put <content>           # Add a memory
memos search <query>          # Hybrid search
memos query <query>           # Full pipeline (search + rerank + graph)
memos intent <query>          # Intent-classified search
memos surface                 # Inject context into current session
memos extract                 # Run decision extraction on transcript
memos reflect                 # Run A-MEM consolidation + feedback
memos timeline <docid>        # Temporal neighborhood
memos causal <docid>          # Causal chain traversal
memos forget <query>          # Decay/remove a memory
memos status                  # Vault statistics
memos doctor                  # Health check
```

## MCP Tools

| Tool | Description |
|------|-------------|
| `memos_put` | Store a memory with metadata |
| `memos_search` | BM25 + vector hybrid search |
| `memos_query` | Full pipeline with intent routing |
| `memos_intent_search` | WHY/WHEN/ENTITY/WHAT classified search |
| `memos_surface` | Context-aware memory injection |
| `memos_timeline` | Temporal neighborhood |
| `memos_causal` | Causal chain traversal |
| `memos_extract` | Decision extraction from transcript |
| `memos_reflect` | A-MEM consolidation |
| `memos_feedback` | Record usefulness of surfaced memories |
| `memos_forget` | Soft decay or hard delete |
| `memos_status` | Vault statistics |

## Roadmap

| Phase | Scope | Status |
|-------|-------|--------|
| 0 — Spec | Architecture + SPEC.md | 🔄 In Progress |
| 1 — Storage | Memvid integration, file format, basic put/get | 📋 Planned |
| 2 — Retrieval | BM25 + HNSW + RRF + rerank | 📋 Planned |
| 3 — Cognitive | Decision extraction, causal graph, A-MEM | 📋 Planned |
| 4 — Lifecycle | Feedback loop, decay, conflict detection | 📋 Planned |
| 5 — Integrations | CLI, MCP stdio, HTTP API | 📋 Planned |
| 6 — Optimizations | Streaming rerank, calibration probes | 📋 Planned |

## References

Memos builds on ideas from:
- [Memvid](https://github.com/memvid/memvid) — single-file vector storage
- [ClawMem](https://github.com/yoloshii/ClawMem) — cognitive memory layer
- [A-MEM](https://arxiv.org/) — self-evolving memory architecture
- [MAGMA](https://arxiv.org/) — multi-graph memory agent
- [Hindsight](https://arxiv.org/) — entity resolution + MPFP traversal
- [QMD](https://github.com/QMD) — hybrid search backend
- [Engram](https://github.com/) — observation dedup + temporal navigation

## License

MIT

## Contributors

See CONTRIBUTORS.md
