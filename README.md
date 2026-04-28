# Janitor (PC Scan)

Safe, transparent Windows-oriented cleanup scanner: Rust workspace with a shared engine, CLI, and a future privilege broker.

## Quick start

```bash
cargo test -p janitor-engine
cargo run -p janitor-cli -- list
cargo run -p janitor-cli -- scan
```

## Layout

| Path | Role |
|------|------|
| `janitor-engine/` | Core types, blacklist, scanners (`temp_dirs`, `recycle_bin`, `browser_cache`) |
| `janitor-cli/` | `janitor` binary — `scan`, `list` |
| `janitor-broker/` | Stub binary for future elevated operations |

## Documentation

- [`FOUNDATION.md`](FOUNDATION.md) — what is implemented today
- [`ARCHITECTURE.md`](ARCHITECTURE.md) — trust model and layers
- [`DOCS/SUMMARY.md`](DOCS/SUMMARY.md) — project status and links (agent workflow)

License: MPL-2.0 (see workspace `Cargo.toml`).
