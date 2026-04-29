# Changelog

## [Unreleased] — Phase 2 in development

### Documentation

- Workspace audit logged in `DOCS/debugs/debug_2026-04-27_audit.md`; `DOCS/SUMMARY.md` / `DOCS/SBOM.md` refreshed (`janitor-ui`, 64 engine tests, `cargo-audit` follow-up).
- `README.md` Testing section aligned with `cargo test --workspace` counts.

### Fixed

- `janitor-cli`: `clippy::print_literal` in table headers (`scan` / `list` output).

### Planned

- Quarantine module (undo-able deletions with audit log)
- Rule engine (TOML-based, signed, versioned)
- SQLite persistence
- Privilege broker (Windows service + IPC)

## [0.1.2] — Tauri UI — 2026-04-28 (current)

### Added
- `janitor-ui` crate: Tauri 2 desktop application
  - Dark-themed UI with live scan results table
  - Summary stats (findings, reclaimable space, risk breakdown, duration)
  - Live search filter + risk-level filter
  - Calls engine directly via Tauri IPC (`run_scan`, `list_scanners`, `engine_version`)
  - Vanilla HTML/CSS/JS — no npm/Node required
  - Placeholder icon (`icons/icon.ico`) — replace with production asset
- `launch.bat` option 7: launch desktop UI (builds if needed)
- `janitor-ui` added to workspace `Cargo.toml`

### Requirements (for UI)
- Windows 10+ with WebView2 runtime (pre-installed on Win11; installer: microsoft.com/en-us/microsoft-edge/webview2)
- Build: `cargo build --release -p janitor-ui`

## [0.1.1] — Phase 1 Enhanced — 2026-04-27 (current)

### Added
- 2 additional scanners: `crash_dumps`, `windows_update` (5 total)
- Enhanced CLI with filtering: `--min-size-mb`, `--category`, `--risk`
- HTML report export (`--html`)
- JSON file export (`--output`)
- Interactive menu system (`launch.bat`)
- About/info command (`janitor about`)
- Comprehensive test suite: **35+ tests** covering all modules

> **[AMENDED 2026-04-27]:** Engine unit tests are now **64** (`cargo test -p janitor-engine` / `--workspace`).
- `.gitignore` for Rust development
- Enhanced `README.md` with quick-start guide

### Improvements
- Better CLI help and usage information
- Color-coded HTML reports (risk levels color-mapped)
- Scanner metadata (description field on all scanners)
- Summary statistics in all output formats

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
