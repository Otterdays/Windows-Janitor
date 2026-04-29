<!-- PRESERVATION RULE: Never delete or replace content. Append or annotate only. -->

# Project summary

_Last reviewed: 2026-04-27 (audit pass)_

## Status

- **Build:** `cargo test --workspace` clean — **64** unit tests in `janitor-engine`; CLI/UI/broker bins compile (those crates have 0 unit tests).
- **Phase 1 (read-only scan):** Five engine scanners + `janitor` CLI + **`janitor-ui`** (Tauri 2 desktop). Broker remains a stub.
- **Git:** Remote [Otterdays/Windows-Janitor](https://github.com/Otterdays/Windows-Janitor); `.gitignore` excludes `target/`, local env, `.claude/settings.local.json`.
- **Audit:** See [`debugs/debug_2026-04-27_audit.md`](debugs/debug_2026-04-27_audit.md) — `cargo-audit` not installed here; run before releases.

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
| [`debugs/debug_2026-04-27_audit.md`](debugs/debug_2026-04-27_audit.md) | Latest audit log |

## Crates

- `janitor-engine` — lib: models, `Scanner` trait, blacklist, five scanners.
- `janitor-cli` — bin `janitor`.
- `janitor-ui` — bin `janitor-ui` (Tauri 2).
- `janitor-broker` — bin `janitor-broker` (placeholder).
