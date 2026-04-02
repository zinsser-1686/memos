# Memos — Technical Specification

**Version**: 0.1.0-draft  
**Status**: 🔄 Draft — not implemented  
**Last Updated**: 2026-04-01

---

## 1. Overview

Memos is an AI agent memory system that combines:

1. **Memvid's `.mv2` single-file storage engine** — HNSW vectors, BM25 full-text, temporal index, append-only frames, crash-safe WAL
2. **ClawMem's cognitive intelligence** — decision extraction, causal graph traversal, A-MEM self-evolution, feedback-driven decay, content-type half-lives

The result is a memory system that is simultaneously **fast** (sub-millisecond retrieval at scale) and **intelligent** (autonomously extracts decisions, maintains causal chains, evolves its own metadata).

---

## 2. Architecture

### 2.1 Layer Map

```
┌────────────────────────────────────────────────────────────┐
│                    Cognitive Layer                          │
│  Decision Extraction │ Causal Graph │ A-MEM │ Feedback     │
│────────────────────────────────────────────────────────────│
│                    Retrieval Layer                         │
│  Hybrid Search │ Intent Classification │ RRF │ Rerank     │
│────────────────────────────────────────────────────────────│
│                    Storage Layer (Memvid)                  │
│  .mv2 File │ HNSW │ BM25 (Tantivy) │ Temporal │ WAL      │
└────────────────────────────────────────────────────────────┘
```

### 2.2 Data Flow

```
Session Transcript
    │
    ▼
┌──────────────────┐
│ Decision Extract │  ← Local LLM (GGUF / API)
│    (on Stop)     │
└────────┬─────────┘
         │
         ▼
┌──────────────────┐     ┌──────────────────┐
│   Causal Edge    │────►│  Memory Node     │
│   Extraction     │     │  (fragment)      │
└──────────────────┘     └────────┬─────────┘
                                  │
                                  ▼
                        ┌──────────────────┐
                        │  A-MEM Enrich    │
                        │  Keywords/Tags/  │
                        │  Context/Links   │
                        └────────┬─────────┘
                                 │
                                 ▼
                        ┌──────────────────┐
                        │  .mv2 Storage    │
                        │  (Memvid)        │
                        └──────────────────┘

User Query
    │
    ▼
┌──────────────────┐
│ Intent Classify  │  ← WHY / WHEN / ENTITY / WHAT
│ (fast heuristics │
│  + LLM refine)   │
└────────┬─────────┘
         │
         ├──────────────────────────────────┐
         │                                  │
         ▼                                  ▼
┌──────────────────┐              ┌──────────────────┐
│   BM25 Search    │              │  Vector Search   │
│   (Tantivy)      │              │   (HNSW)         │
└────────┬─────────┘              └────────┬─────────┘
         │                                  │
         └──────────────┬───────────────────┘
                        │
                        ▼
              ┌──────────────────┐
              │      RRF        │
              │ (Rank Fusion)   │
              └────────┬─────────┘
                       │
                       ▼
              ┌──────────────────┐
              │ Cross-Encoder    │
              │   Rerank         │
              └────────┬─────────┘
                       │
                       ▼
              ┌──────────────────┐
              │  Intent-Aware    │
              │  Graph Expansion │
              │  (WHY/ENTITY)    │
              └────────┬─────────┘
                       │
                       ▼
              ┌──────────────────┐
              │  Composite       │
              │  Scoring         │
              │  (A-MEM signals) │
              └────────┬─────────┘
                       │
                       ▼
              ┌──────────────────┐
              │  MMR Diversity   │
              │    Filter        │
              └────────┬─────────┘
                       │
                       ▼
              ┌──────────────────┐
              │  Top-K Results   │
              └──────────────────┘
```

---

## 3. Storage Layer — Memvid Integration

### 3.1 File Format

`.mv2` file structure:

```
┌─────────────────────────────────────────┐
│ Header (4KB)                           │  Magic, version, capacity
├─────────────────────────────────────────┤
│ Embedded WAL (1–64MB)                   │  Crash recovery, immutable frames
├─────────────────────────────────────────┤
│ Data Segments                           │  Compressed frames
├─────────────────────────────────────────┤
│ Lex Index (Tantivy FTS)                │  BM25 full-text
├─────────────────────────────────────────┤
│ Vec Index (HNSW)                       │  Vector similarity
├─────────────────────────────────────────┤
│ Time Index                              │  Chronological ordering
├─────────────────────────────────────────┤
│ TOC (Footer)                            │  Segment offsets
└─────────────────────────────────────────┘
```

### 3.2 Memory Frame Schema

Each memory is stored as an **immutable frame** with this structure:

```json
{
  "frame_id": "f_abc123",
  "type": "decision | hub | research | handoff | note | progress | antipattern",
  "title": "string",
  "content": "string (raw text)",
  "fragment": "string (chunked semantic unit)",
  "keywords": ["string"],
  "tags": ["string"],
  "context_desc": "string",
  "uri": "string (stable address)",
  "source_session": "string",
  "source_context": "string",
  "observed_at": "ISO8601",
  "created_at": "ISO8601",
  "updated_at": "ISO8601",
  "confidence": 0.85,
  "access_count": 0,
  "last_accessed": null,
  "decay_started": null,
  "lifecycle": "active | archived | snoozed | forgotten",
  "snooze_until": null,
  "pin_priority": 0.0,
  "replaces": ["frame_id"],
  "replaced_by": null,
  "vector": [0.0, ...],
  "metadata": {}
}
```

### 3.3 Relations Table

Causal/semantic/emporal edges are stored in a **sidecar SQLite file** (not in `.mv2`):

```sql
CREATE TABLE relations (
  id          INTEGER PRIMARY KEY,
  source_id   TEXT NOT NULL,
  target_id   TEXT NOT NULL,
  rel_type    TEXT NOT NULL,  -- 'cause' | 'effect' | 'semantic' | 'temporal' | 'contradicts' | 'supports'
  confidence  REAL DEFAULT 0.0,
  created_at  TEXT,
  UNIQUE(source_id, target_id, rel_type)
);

CREATE INDEX idx_relations_source ON relations(source_id);
CREATE INDEX idx_relations_target ON relations(target_id);
CREATE INDEX idx_relations_type  ON relations(rel_type);
```

### 3.4 Entity Table

```sql
CREATE TABLE entities (
  id          INTEGER PRIMARY KEY,
  name        TEXT NOT NULL UNIQUE,
  entity_type TEXT,           -- 'person' | 'project' | 'tool' | 'concept' | 'other'
  canonical   TEXT,           -- canonical form
  created_at  TEXT,
  updated_at  TEXT
);
```

---

## 4. Retrieval Layer

### 4.1 Search Pipeline

#### Stage 1: BM25 Probe

Tantivy full-text search on `content` + `title` + `keywords` fields.

```rust
bm25_search(query: &str, top_k: usize) -> Vec<Hit>
```

#### Stage 2: Vector Search

HNSW search on `fragment` embeddings (dimension: 384 or 768 depending on model).

```rust
vec_search(query_vector: &[f32], top_k: usize) -> Vec<Hit>
```

#### Stage 3: Intent Classification

```rust
enum QueryIntent {
    WHY,     // "why did we decide X"
    WHEN,    // "when was Y discussed"
    ENTITY,  // "who worked on Z"
    WHAT,    // general recall
}

fn classify_intent(query: &str) -> (QueryIntent, f32)
// Returns (intent, confidence)
// Fast heuristics first, LLM refine only if confidence < 0.8
```

#### Stage 4: Reciprocal Rank Fusion

```rust
fn rrf_merge(bm25_hits: Vec<Hit>, vec_hits: Vec<Hit>, k: usize) -> Vec<Hit>
    // RRF score = Σ 1/(k + rank_i)
    // k = 60 (standard)
```

#### Stage 5: Cross-Encoder Rerank

```rust
fn rerank(query: &str, candidates: Vec<Hit>, top_k: usize) -> Vec<Hit>
    // Uses cross-encoder model (or LLM-as-judge fallback)
    // 4000 char context per doc
```

#### Stage 6: Intent-Aware Graph Expansion

For `WHY` and `ENTITY` intents:
- Start from top-10 baseline results as anchor nodes
- For each node: traverse outgoing `cause`/`effect` edges
- Score transitions: `λ1·structure + λ2·semantic_affinity`
- Apply decay: `new_score = parent_score * γ + transition_score`
- Keep top-k via beam search

#### Stage 7: Composite Scoring (A-MEM Signals)

```rust
fn composite_score(hit: &Hit, signals: &ScoringSignals) -> f32 {
    let recency     = hit.recency_decay();       // Ebbinghaus curve
    let confidence  = hit.confidence;           // baseline by content_type
    let quality      = hit.quality_multiplier(); // length norm + structure
    let coactivation = hit.coactivation_boost(); // frequently co-accessed
    let pin          = hit.pin_boost();          // +0.3 if pinned

    let base = search * 0.5 + recency * 0.25 + confidence * 0.25;
    base * quality * coactivation + pin
}
```

#### Stage 8: MMR Diversity Filter

```rust
fn mmr_filter(candidates: Vec<Hit>, target: usize, lambda: f32) -> Vec<Hit>
    // Demote candidates with Jaccard bigram similarity > 0.6 to top hit
```

### 4.2 Retrieval Configuration

```yaml
retrieval:
  bm25:
    k1: 1.5
    b: 0.75

  vector:
    hnsw:
      m: 16
      ef_construction: 200
      ef_search: 100

  rrf:
    k: 60

  rerank:
    cross_encoder_model: "cross-encoder/ms-marco-MiniLM-L-12-v2"
    context_chars: 4000
    batch_size: 8

  mmr:
    enabled: true
    lambda: 0.7
    diversity_threshold: 0.6

  intent:
    heuristic_threshold: 0.8
    llm_model: "qwen3-1.7b"
```

---

## 5. Cognitive Layer

### 5.1 Decision Extraction

Triggered on `session_stop` hook. Input: raw transcript. Output: structured observations.

```rust
struct ExtractedDecision {
    decision_type: DecisionType,  // decision | observation | preference | fact
    title: String,
    facts: Vec<Fact>,
    narrative: String,
    contradicts: Vec<String>,    // frame_ids of superseded decisions
}

fn extract_decisions(transcript: &str) -> Vec<ExtractedDecision>
    // Uses local GGUF model (qwen3-1.7b or similar)
    // Falls back to regex patterns if model unavailable
    // Contradiction detection: if prior decision has opposite polarity, flag
```

**Extraction flow**:
1. Split transcript into user/assistant turns
2. For each assistant turn with tool calls or conclusions:
   - Prompt LLM: "Extract any decisions, facts, or observations from this turn"
   - Parse structured output
3. For each extracted decision:
   - Compare against prior decisions for contradictions
   - If contradiction detected: set `replaces` on new, lower `confidence` on old (-0.25)

### 5.2 Causal Graph

```rust
enum RelationType {
    Cause,      // A caused B
    Effect,     // B was caused by A
    Semantic,   // A is semantically related to B
    Temporal,   // A occurred before/after B
    Contradicts,// A contradicts B
    Supports,   // A supports B
}

struct Relation {
    source_id: String,
    target_id: String,
    rel_type: RelationType,
    confidence: f32,
}

fn build_causal_edges(decisions: &[ExtractedDecision]) -> Vec<Relation>
    // LLM-inferred cause→effect pairs
    // Confidence ≥ 0.6 required
```

### 5.3 A-MEM (Adaptive Memory Evolution)

On each memory write:

```rust
fn enrich_memory(memory: &mut MemoryFrame) {
    // 1. Extract keywords (3–7 specific terms from content)
    memory.keywords = extract_keywords(memory.content, top_k=7);

    // 2. Extract tags (3–5 broad categories)
    memory.tags = extract_tags(memory.content, top_k=5);

    // 3. Generate context description (1–2 sentences)
    memory.context_desc = summarize(memory.content);

    // 4. Find related memories → create semantic links
    let related = vec_search(memory.vector, top_k=5);
    for rel in related {
        if rel.similarity > 0.7 {
            create_relation(memory.id, rel.id, Semantic, rel.similarity);
        }
    }
}
```

### 5.4 Feedback Loop

On session stop, after retrieval results were surfaced:

```rust
fn process_feedback(surfaced: Vec<FrameId>, referenced: Vec<FrameId>) {
    // Referenced = agent actually used the memory in tools/code/output
    // Surfaced = shown in context but not necessarily used

    for frame_id in referenced {
        boost(frame_id, +0.05);
        access_count[frame_id]++;
    }

    for frame_id in surfaced {
        if frame_id not in referenced {
            // Shown but ignored — minor decay signal
            decay(frame_id, -0.01);
        }
    }
}
```

### 5.5 Content-Type Half-Lives

| Type | Baseline | Half-life | Decay |
|------|----------|-----------|-------|
| `decision` | 0.85 | ∞ | Never |
| `hub` | 0.80 | ∞ | Never |
| `antipattern` | 0.75 | ∞ | Never |
| `research` | 0.70 | 90d | Linear |
| `project` | 0.65 | 120d | Linear |
| `handoff` | 0.60 | 30d | Linear |
| `progress` | 0.50 | 45d | Linear |
| `note` | 0.50 | 60d | Linear |

```rust
fn recency_decay(frame: &MemoryFrame) -> f32 {
    let age_days = now() - frame.last_accessed;
    let half_life = frame.type_half_life();
    if half_life == INFINITY { return 1.0; }
    (0.5_f32).powf(age_days as f32 / half_life as f32)
}
```

### 5.6 Conflict Detection

When extracting a new decision:

```rust
fn detect_conflicts(new: &ExtractedDecision, existing: Vec<&MemoryFrame>) -> Vec<Conflict> {
    // 1. Find same-topic prior decisions
    let candidates = bm25_search(new.title, top_k=10);

    // 2. For each candidate:
    for prior in candidates {
        if prior.type != "decision" { continue; }
        if prior.title == new.title { continue; }

        // Check for negation patterns
        if contains_negation(prior.narrative) && contains_negation(new.narrative) {
            if extracted_value_changed(prior, new) {
                conflicts.push(Conflict {
                    new_id: new.id,
                    old_id: prior.id,
                    severity: 0.25,  // Decay old by this amount
                });
            }
        }
    }
}
```

---

## 6. Lifecycle Management

### 6.1 Memory States

```
active ──► archived ──► forgotten
   │
   └──► snoozed ──► active
   │
   └──► forgotten
```

### 6.2 Reflect Job (Sleep Cycle)

Triggered nightly or on schedule:

```rust
fn reflect(vault: &Vault) {
    // Phase 1: Tidy — detect stale patterns
    detect_stale_content(vault);

    // Phase 2: Decay — apply recency decay to all frames
    apply_recency_decay(vault);

    // Phase 3: Govern — enforce per-type capacity limits
    enforce_capacity_limits(vault);

    // Phase 4: Consolidate — find duplicate/contradictory clusters
    consolidate_clusters(vault);
}
```

### 6.3 Snooze

Temporarily suppress a memory from surfacing without deleting:

```rust
fn snooze(frame_id: &str, until: DateTime) {
    set_state(frame_id, Lifecycle::Snoozed);
    set_field(frame_id, "snooze_until", until);
}
```

### 6.4 Pin

Boost a memory's priority:

```rust
fn pin(frame_id: &str, priority: f32) {
    set_field(frame_id, "pin_priority", priority);
}
```

---

## 7. Integration Interfaces

### 7.1 Rust SDK

```rust
use memos::{Memos, PutOptions, SearchRequest};

let mut memos = Memos::open("vault.mv2")?;

memos.put_with_options(
    b"Use 2μm lasers for copper welding",
    PutOptions::default()
        .title("2μm for copper")
        .r#type("decision")
        .tags(&["welding", "copper", "2μm"])
)?;

let results = memos.search(SearchRequest {
    query: "copper welding".into(),
    top_k: 10,
    intent: Some(Intent::What),
    ..Default::default()
})?;
```

### 7.2 MCP Tools

```json
{
  "memos_search": {
    "description": "Hybrid BM25 + vector search",
    "input": {
      "query": "string",
      "top_k": "number (default 10)",
      "intent_hint": "WHY | WHEN | ENTITY | WHAT (optional)"
    }
  },
  "memos_intent_search": {
    "description": "Intent-classified causal search",
    "input": {
      "query": "string",
      "top_k": "number (default 10)"
    }
  },
  "memos_surface": {
    "description": "Context-aware injection for current session",
    "input": {
      "task": "string (optional context)",
      "recent_turns": "number (default 5)"
    }
  },
  "memos_extract": {
    "description": "Decision extraction from transcript",
    "input": {
      "transcript": "string"
    }
  },
  "memos_causal": {
    "description": "Multi-hop causal chain traversal",
    "input": {
      "doc_id": "string",
      "max_hops": "number (default 3)"
    }
  },
  "memos_forget": {
    "description": "Soft decay or hard delete",
    "input": {
      "query": "string",
      "mode": "soft | hard"
    }
  }
}
```

### 7.3 CLI Commands

```bash
memos init [--path vault.mv2]
memos put <content> [--title T] [--type TYPE] [--tags t1,t2]
memos get <docid>
memos search <query> [--top-k N] [--intent WHY|WHEN|ENTITY|WHAT]
memos query <query>        # Full pipeline
memos intent <query>       # Intent-classified
memos timeline <docid> [--before N] [--after N]
memos causal <docid> [--hops N]
memos extract [--session-id ID]
memos reflect [--dry-run]
memos feedback <docid> <helpful|neutral|harmful>
memos forget <query> [--hard]
memos pin <docid> [--priority N]
memos snooze <docid> --until YYYY-MM-DD
memos status
memos doctor
```

---

## 8. Configuration

### 8.1 Default Configuration

```yaml
# memos.yaml

vault:
  path: "memos.mv2"
  relations_db: "memos.db"  # SQLite sidecar for relations/entities

embedding:
  provider: "local"  # local | openai | voyage | jina | ollama
  model: "BAAI/bge-small-en-v1.5"
  dimension: 384
  local:
    model_path: "~/.cache/memos/models/bge-small-en-v1.5"
    provider: "llama-server"
    url: "http://localhost:8088"

llm:
  provider: "local"
  model: "qwen3-1.7b-q4_k_m"
  local:
    url: "http://localhost:8089"
    no_think: true  # Append /no_think to prompts

reranker:
  provider: "local"
  model: "cross-encoder/ms-marco-MiniLM-L-12-v2"
  local:
    url: "http://localhost:8090"

retrieval:
  bm25:
    k1: 1.5
    b: 0.75
  vector:
    hnsw:
      m: 16
      ef_construction: 200
      ef_search: 100
  rrf:
    k: 60
  rerank:
    context_chars: 4000
    batch_size: 8
  mmr:
    enabled: true
    lambda: 0.7
    diversity_threshold: 0.6

lifecycle:
  reflect:
    enabled: true
    interval_hours: 24
  decay:
    enabled: true
  governance:
    max_memory: 350
    per_type_limits:
      identity: null     # unlimited
      emotion: 50
      knowledge: 250
      event: 50
  consolidation:
    enabled: true
    deduplication_window_minutes: 30
```

### 8.2 Environment Variables

| Variable | Default | Description |
|----------|---------|-------------|
| `MEMOS_VAULT_PATH` | `memos.mv2` | Vault file path |
| `MEMOS_EMBED_URL` | `http://localhost:8088` | Embedding server |
| `MEMOS_LLM_URL` | `http://localhost:8089` | LLM server |
| `MEMOS_RERANK_URL` | `http://localhost:8090` | Reranker server |
| `MEMOS_CONFIG` | `memos.yaml` | Config file path |

---

## 9. Project Structure

```
memos/
├── src/
│   ├── lib.rs              # Public crate root
│   ├── main.rs             # CLI entrypoint
│   ├── storage/            # Memvid integration
│   │   ├── mod.rs
│   │   ├── vault.rs        # .mv2 file operations
│   │   ├── frame.rs       # Memory frame schema
│   │   └── relations.rs    # Sidecar SQLite
│   ├── retrieval/          # Search pipeline
│   │   ├── mod.rs
│   │   ├── bm25.rs
│   │   ├── vector.rs
│   │   ├── rrf.rs
│   │   ├── rerank.rs
│   │   └── intent.rs
│   ├── cognitive/          # Intelligence layer
│   │   ├── mod.rs
│   │   ├── extract.rs      # Decision extraction
│   │   ├── graph.rs        # Causal graph
│   │   ├── amem.rs         # A-MEM evolution
│   │   ├── feedback.rs     # Feedback loop
│   │   └── conflict.rs     # Conflict detection
│   ├── lifecycle/          # Memory management
│   │   ├── mod.rs
│   │   ├── reflect.rs
│   │   ├── decay.rs
│   │   └── governance.rs
│   ├── hooks/              # Session lifecycle hooks
│   │   ├── mod.rs
│   │   └── events.rs
│   ├── api/                # MCP + HTTP interfaces
│   │   ├── mod.rs
│   │   ├── mcp.rs
│   │   └── http.rs
│   └── config.rs
├── tests/
│   ├── integration/
│   └── unit/
├── examples/
│   ├── basic.rs
│   ├── cognitive.rs
│   └── mcp.rs
├── docs/
│   ├── SPEC.md
│   ├── ARCHITECTURE.md
│   └── INTEGRATION.md
├── SPEC.md
└── README.md
```

---

## 10. Open Questions

| # | Question | Status |
|---|----------|--------|
| 1 | Cross-encoder reranking — local GGUF or API? | 📋 Local GGUF preferred |
| 2 | Conflict detection — LLM or pattern-based? | 📋 LLM-first, regex fallback |
| 3 | Entity extraction — dedicated model or LLM? | 📋 LLM with NER hints |
| 4 | Multiple vaults (work vs personal)? | 📋 Phase 2 |
| 5 | Encryption for .mv2 files? | 📋 Optional v2 feature |

---

## 11. Dependencies

### Rust Crates (Key)

| Crate | Version | Purpose |
|-------|---------|---------|
| `memvid-core` | 2.0 | Storage engine |
| `tantivy` | 0.22 | BM25 full-text search |
| `ndarray` | 0.16 | Vector math |
| `serde` | 1.0 | Serialization |
| `rusqlite` | 0.32 | Relations/entities sidecar |
| `tokio` | 1.0 | Async runtime |
| `clap` | 4.0 | CLI argument parsing |
| `tracing` | 0.1 | Logging |

### External Services (Optional)

| Service | Purpose | Required |
|---------|---------|---------|
| `llama-server` (port 8088) | Embedding | No (falls back to in-process) |
| `llama-server` (port 8089) | LLM inference | No (falls back to in-process) |
| `llama-server` (port 8090) | Reranker | No (falls back to in-process) |

---

*Last updated: 2026-04-01*
