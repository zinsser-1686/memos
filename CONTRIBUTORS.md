# Contributors

## Project Lead

- **Zinsser Liu** (@zinsser-1686) — Project creator and lead maintainer

## Acknowledgments

Memos builds on the pioneering work of:

- **Memvid** ([@memvid](https://github.com/memvid)) — Single-file vector storage engine
- **ClawMem** ([@yoloshii](https://github.com/yoloshii)) — Cognitive memory layer with A-MEM
- **OpenClaw** — Agent framework inspiration
- **Claude Code** — Product inspiration

## Contributing

Contributions are welcome! Please see [SPEC.md](docs/SPEC.md) for the detailed technical specification.

### Development Setup

```bash
# Install Rust 1.85+
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Clone and build
git clone https://github.com/zinsser-1686/memos.git
cd memos
cargo build --release

# Run tests
cargo test

# Run with logging
RUST_LOG=debug cargo run -- search "test query"
```

### Key Areas for Contribution

| Area | Description | Status |
|------|-------------|--------|
| Decision extraction | LLM-based extraction from transcripts | 🔴 Not implemented |
| Causal inference | LLM-based cause→effect edge building | 🔴 Not implemented |
| A-MEM enrichment | Keyword/tag/context generation | 🟡 Stub only |
| Cross-encoder rerank | Integration with local reranker | 🟡 Stub only |
| Conflict detection | Negation pattern matching | 🟡 Basic patterns |
| Reflect cycle | Full sleep cycle implementation | 🔴 Not implemented |
| MCP server | JSON-RPC over stdio | 🔴 Not implemented |
| HTTP API | REST endpoints | 🔴 Skeleton only |
| Tests | Integration tests | 🔴 Unit tests only |

### Code Style

- Use `cargo fmt` before committing
- Run `cargo clippy` to check for warnings
- Write unit tests for new functions
- Document public APIs with doc comments

## License

MIT
