# Changelog

## [Unreleased]

### Added

- `DOCS/` agent workflow files (`SUMMARY`, `SBOM`, `SCRATCHPAD`, `STYLE_GUIDE`, `My_Thoughts`, `CHANGELOG` index, `ARCHITECTURE` pointer, `debugs/`).
- Root `README.md` and `.gitignore` (Rust `target/`, local env, `.claude/settings.local.json`).

### Documentation

- **[AMENDED 2026-04-27]:** The `[0.1.0]` section listed scanners and CLI as “not yet built”; Phase 1 scanners and `janitor-cli` are implemented (`FOUNDATION.md`). Broker remains a stub; Phase 2 items unchanged.

## [0.1.0] — Foundation — 2026-04-27

### Added
- Workspace skeleton (`janitor-engine`, `janitor-broker`, `janitor-cli` crates declared)
- `janitor-engine` core types: `Finding` (builder pattern), `ScanContext`, `ScanResult`, `Category` (18 variants), `RiskLevel`, `TargetKind`
- `Scanner` trait — stateless, `Send + Sync`, returns `Vec<Finding>`
- `blacklist` module — `is_path_safe()`, case-insensitive prefix matching, static never-touch list
- `error` module — `Error` enum, `Result<T>` alias
- Unit tests: finding builder, risk ordering, scan result aggregation, blacklist

### Not yet built

> **[AMENDED 2026-04-27]:** The following bullets were planning notes at release time. **Superseded:** `temp_dirs`, `recycle_bin`, and `browser_cache` scanners exist; `janitor-cli` exists. **`janitor-broker`** is present as a minimal binary (stub; no Windows service yet).

- Any scanner implementations (temp dirs, recycle bin, browser cache)
- `janitor-cli` binary
- `janitor-broker` binary
- Rule engine, quarantine, persistence, Tauri UI
