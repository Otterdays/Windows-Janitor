# Janitor Foundation Layer

_Last updated: 2026-04-27_

## Status

Foundation + Phase 1 scanners + CLI are **complete** and building clean. Broker and UI are Phase 2.

## What's Built

**Core engine library** (`janitor-engine` crate) — pure Rust, no UI, no Windows dependencies except where necessary.

Key external dependencies (declared in `janitor-engine/Cargo.toml`):
- `serde` / `serde_json` — serialization
- `uuid` — finding/scan IDs
- `chrono` — ISO 8601 timestamps on findings
- `thiserror` — error derivation
- `walkdir` — directory traversal in scanners
- `rayon` — parallel scanner execution
- `blake3` — file hashing (reserved for duplicate detection, Phase 2)

### Modules

**`models.rs`** — Core data types
- `ScanContext` — passed to every scanner; constructed via `ScanContext::new()` or `Default`
- `Finding` — a single junk item; builder pattern (`Finding::new(…).with_size().with_age().with_reason().with_confidence()`)
- `ScanResult` — aggregated scan output; holds `findings: Vec<Finding>`, `errors: Vec<String>`, `duration_ms`
- `Category` — 18-variant enum (TempFiles, RecycleBin, BrowserCache, …, Other)
- `RiskLevel` — Low / Medium / High; implements `Ord` (Low < Medium < High)
- `TargetKind` — File / Directory / RegistryKey / RegistryValue / Service / ScheduledTask

**`scanner.rs`** — Scanner trait
- Every scanner implements this
- Independent, stateless, runs in parallel via rayon
- Method: `scan(&self, ctx: &ScanContext) -> Result<Vec<Finding>>`
- Optional overrides: `requires_elevation()`, `enabled_by_default()`, `description()`

**`blacklist.rs`** — Hard blacklist (safety boundary)
- `is_path_safe(path: &Path) -> bool` — returns true if safe to touch
- `is_blacklisted(path: &Path) -> bool` — inverse; used in tests
- Static never-touch list: WinSxS, System32, SysWOW64, Servicing, Assembly, pagefile/hiberfil/swapfile, Boot
- Normalizes paths (resolves `..`) before checking — resistant to path traversal
- `C:\Windows\Temp` is NOT blacklisted (scanners may report it)

**`error.rs`** — Error types
- `Error` enum: Io, BlacklistViolation, Scanner, InvalidRule, ScanContext, Classification, Quarantine, Registry, PermissionDenied, Serialization
- `Result<T>` shorthand

**`lib.rs`** — Public API surface
- Re-exports: `Error`, `Result`, `Category`, `Finding`, `RiskLevel`, `ScanContext`, `ScanResult`, `TargetKind`, `Scanner`, `all_scanners`

**`scanners/mod.rs`** — Scanner registry
- `all_scanners() -> Vec<Box<dyn Scanner>>` — returns all Phase 1 scanners
- Sub-modules: `temp_dirs`, `recycle_bin`, `browser_cache`

**`scanners/temp_dirs.rs`** — `TempDirScanner`
- Scans `%TEMP%`, `%TMP%`, `%LOCALAPPDATA%\Temp`, `C:\Windows\Temp`
- Skips files with age 0 (in use); confidence scales with age (60%→95%)
- Results sorted by confidence descending

**`scanners/recycle_bin.rs`** — `RecycleBinScanner`
- Walks `C:\$Recycle.Bin` (and D:, E:, etc.)
- Skips `$I` metadata files; reports `$R` content files
- Results sorted by size descending; confidence 0.99

**`scanners/browser_cache.rs`** — `BrowserCacheScanner`
- Covers Chrome, Edge, Firefox, Brave, Opera (including multi-profile)
- Reports one finding per cache root (directory summary), not per file
- Skips empty cache dirs (0 bytes)
- Results sorted by size descending

## How Scanners Work

```rust
pub trait Scanner: Send + Sync {
    fn id(&self) -> &'static str;                         // "temp_dirs"
    fn name(&self) -> &'static str;                       // "Temporary Files"
    fn requires_elevation(&self) -> bool;                 // false for most
    fn scan(&self, ctx: &ScanContext) -> Result<Vec<Finding>>;
}
```

Scanners:
- Never call blacklist directly (future middleware will check)
- Return findings with full detail (path, size, age, reason, confidence)
- Run in parallel (rayon thread pool, `cpu_count - 1`)
- Are unit-testable in isolation

## Safety Architecture

1. **Blacklist is enforced last** — defense-in-depth. Even if a scanner yields something on the blacklist, it should be rejected before reaching UI.
2. **Findings are immutable** — built via builder pattern, snapshots.
3. **Rules separate from code** — (Phase 2) TOML-based, signed, versioned.
4. **Confidence scores** — heuristic findings are marked as suggestions, not certain.

## Testing

```bash
cargo test                        # all crates
cargo test -p janitor-engine      # engine only
```

Test count: **30+ tests** across all modules. Coverage includes:

| Module | Tests |
|---|---|
| `blacklist` | WinSxS, System32, SysWOW64, Servicing, Assembly, kernel files, Boot, safe paths, path traversal, case insensitivity, `is_blacklisted` inverse |
| `models` | Finding builder, unique IDs, confidence clamping, RFC3339 timestamp, RiskLevel ordering + display, all Category display variants, ScanContext defaults + unique IDs, ScanResult aggregation |
| `scanner` | Trait compiles and runs via mock |
| `scanners/mod` | Count, unique IDs, non-empty metadata, known IDs present, no elevation required |
| `scanners/temp_dirs` | ID/name, no elevation, skips fresh files, nonexistent path, empty dir, no duplicate temp paths, sort order, blacklisted paths skipped |
| `scanners/recycle_bin` | ID/name, no elevation, nonexistent path, skips `$I` files, size sort, RecycleBin category |
| `scanners/browser_cache` | ID/name, no elevation, nonexistent path, one finding per root, empty dir skipped, size accuracy, size sort |

## CLI Usage

```bash
# Human-readable report
cargo run -p janitor-cli -- scan

# JSON output (pipe to file or jq)
cargo run -p janitor-cli -- scan --json

# Single scanner only
cargo run -p janitor-cli -- scan --scanner temp_dirs

# Include developer caches (npm, cargo, pip)
cargo run -p janitor-cli -- scan --dev-caches

# List available scanners
cargo run -p janitor-cli -- list
```

All scans are **read-only** — no files are modified.

## Phase 2: Next Steps

- `janitor-engine/src/scanners/windows_update.rs` — WinSxS cleanup candidates (DISM-only)
- `janitor-engine/src/scanners/crash_dumps.rs`
- `janitor-engine/src/scanners/thumbnail_cache.rs`
- Rule engine (`rules/`) — TOML-based, signed
- SQLite persistence (`persistence/`)
- Quarantine module (`quarantine/`)
- Privilege broker (`janitor-broker`) — named pipe, Windows service
- Tauri UI (`janitor-ui`)
