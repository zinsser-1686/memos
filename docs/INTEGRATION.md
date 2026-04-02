# Integration Guide

## Rust SDK

```rust
use memos::{Vault, PutOptions, SearchRequest, QueryIntent};

fn main() -> anyhow::Result<()> {
    // Open or create vault
    let mut vault = Vault::open("my-vault.mv2")?;

    // Store a memory
    vault.put_with_options(
        b"We decided to use 2μm lasers for copper welding",
        PutOptions::default()
            .title("2μm for copper")
            .memory_type("decision")
            .tags(vec!["welding".to_string(), "copper".to_string()]),
    )?;

    // Search
    let results = vault.search(SearchRequest {
        query: "copper welding decisions".to_string(),
        top_k: 10,
        intent: Some(QueryIntent::Why),
        ..Default::default()
    })?;

    for hit in results {
        println!("{}: {} (score: {:.3})", hit.frame_id, hit.title, hit.score);
    }

    Ok(())
}
```

## CLI

```bash
# Initialize vault
memos init my-vault.mv2

# Add a memory
memos put "Use 2μm lasers for copper welding" \
    --title "2μm for copper" \
    --type decision \
    --tags welding,copper

# Search
memos search "copper welding" --top-k 10

# Full pipeline
memos query "why did we choose 2μm" --top-k 5

# Intent-classified
memos intent "who worked on this" --top-k 5

# Timeline
memos timeline f_abc123 --before 3 --after 3

# Causal chain
memos causal f_abc123 --hops 3

# Run consolidation
memos reflect

# Status
memos status
```

## MCP (Claude Code, OpenClaw)

Add to your MCP config:

```json
{
  "mcpServers": {
    "memos": {
      "command": "memos",
      "args": ["mcp"]
    }
  }
}
```

## HTTP API

```bash
# Start server
memos serve --port 7438

# Health check
curl http://localhost:7438/health

# Search
curl -X POST http://localhost:7438/v1/search \
  -H 'Content-Type: application/json' \
  -d '{"query": "copper welding", "top_k": 10}'

# Store
curl -X POST http://localhost:7438/v1/memories \
  -H 'Content-Type: application/json' \
  -d '{"content": "Use 2μm for copper", "type": "decision"}'
```

## OpenClaw Integration

```javascript
// In your OpenClaw workspace
// Enable Memos as memory backend:

// 1. Install memos
// npm install memos

// 2. Configure in openclaw.json
{
  "plugins": {
    "slots": {
      "memory": "memos"
    }
  },
  "pluginsConfig": {
    "memos": {
      "vaultPath": "~/.cache/memos/vault.mv2"
    }
  }
}
```

## Claude Code Integration

```json
// In ~/.claude/settings.json
{
  "mcpServers": {
    "memos": {
      "command": "memos",
      "args": ["mcp"]
    }
  }
}
```
