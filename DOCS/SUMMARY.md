<!-- PRESERVATION RULE: Never delete or replace content. Append or annotate only. -->

# Project summary

_Last reviewed: 2026-04-27_

## Status

- **Build:** `cargo test` clean — 60 unit tests in `janitor-engine`; CLI and broker bins compile.
- **Phase 1 (read-only scan):** Engine scanners + `janitor` CLI are in place. Broker is a stub; no Tauri UI yet.
- **Git:** Repository initialized with `.gitignore` (excludes `target/`, local Claude settings).

## Quick links

| Doc | Purpose |
|-----|---------|
| [`../README.md`](../README.md) | User-facing overview |
| [`../FOUNDATION.md`](../FOUNDATION.md) | Implementation detail |
| [`../ARCHITECTURE.md`](../ARCHITECTURE.md) | Security / layering model |
| [`../PROJECT_LAYOUT.md`](../PROJECT_LAYOUT.md) | Directory map (see amendment note at top) |
| [`../CHANGELOG.md`](../CHANGELOG.md) | Version history |
| [`SBOM.md`](SBOM.md) | Dependency inventory |
| [`SCRATCHPAD.md`](SCRATCHPAD.md) | Active tasks / handoff |
| [`STYLE_GUIDE.md`](STYLE_GUIDE.md) | Conventions |

## Crates

- `janitor-engine` — lib: models, `Scanner` trait, blacklist, scanners.
- `janitor-cli` — bin `janitor`.
- `janitor-broker` — bin `janitor-broker` (placeholder).
