# Janitor Project Layout

_Last updated: 2026-04-27 (Phase 1 enhanced)_

> **[AMENDED 2026-04-27]:** The ASCII tree still labels `scanners/`, `janitor-cli`, and `janitor-broker` as `[TODO]` / “no src yet”; those sources **exist** (Phase 1 scan + CLI; broker stub). Prefer `DOCS/SUMMARY.md` and this file’s checklist below over the tree markers for those paths.

Legend: ✓ = exists and compiles | [TODO] = not yet created

```
janitor/                              # Workspace root
├── Cargo.toml                    ✓   # Workspace manifest (members: engine, broker, cli)
├── Cargo.lock                    ✓   # Generated
│
├── janitor-engine/               ✓   # Core scanning engine (lib)
│   ├── Cargo.toml                ✓
│   └── src/
│       ├── lib.rs                ✓   # Public API (re-exports Error, Result, Category, Finding, ScanContext, Scanner)
│       ├── error.rs              ✓   # Error types
│       ├── models.rs             ✓   # Finding, ScanResult, ScanContext, Category, RiskLevel, TargetKind
│       ├── scanner.rs            ✓   # Scanner trait
│       ├── blacklist.rs          ✓   # Safety boundary (never-touch paths)
│       ├── scanners/             ✓   Scanner implementations (5 total)
│       │   ├── mod.rs                ✓
│       │   ├── temp_dirs.rs          ✓   Temp directory scanner
│       │   ├── recycle_bin.rs        ✓   Recycle Bin analysis
│       │   ├── browser_cache.rs      ✓   Browser cache (Chrome, Edge, Firefox, Brave, Opera)
│       │   ├── crash_dumps.rs        ✓   .dmp and .log files
│       │   ├── windows_update.rs     ✓   Windows Update leftovers
│       │   └── (Phase 2: more scanners)
│       ├── rules/                [TODO] Rule engine (Phase 2)
│       │   ├── mod.rs
│       │   ├── parser.rs              # TOML → rule struct
│       │   └── store.rs               # Load / validate / sign rules
│       ├── quarantine/           [TODO] Quarantine module (Phase 2)
│       │   ├── mod.rs
│       │   ├── manifest.rs
│       │   ├── undo_log.rs
│       │   └── retention.rs
│       └── persistence/          [TODO] SQLite schema (Phase 2)
│           ├── mod.rs
│           ├── migrations.rs
│           └── schema.rs
│
├── janitor-broker/               [TODO] Privilege broker (Windows service, Phase 2)
│   ├── Cargo.toml                ✓   # Crate declared in workspace, no src yet
│   └── src/
│       └── main.rs                    # Named pipe listener
│
├── janitor-cli/                  [TODO] CLI binary (Phase 1 MVP)
│   ├── Cargo.toml                ✓   # Crate declared in workspace, no src yet
│   └── src/
│       └── main.rs                    # Basic CLI + JSON/HTML report generation
│
└── janitor-ui/                   ✓   Tauri 2 desktop UI
    ├── Cargo.toml                ✓
    ├── build.rs                  ✓
    ├── tauri.conf.json           ✓
    ├── icons/
    │   └── icon.ico              ✓   32x32 placeholder (replace for production)
    ├── dist/                     ✓   Vanilla HTML/CSS/JS frontend
    │   ├── index.html            ✓   App shell
    │   ├── style.css             ✓   Dark theme
    │   └── app.js                ✓   Tauri IPC calls, live filter, table render
    └── src/
        └── main.rs               ✓   Tauri commands: list_scanners, run_scan, engine_version
```

## Crate Dependencies

```
janitor-ui (Tauri)
    ↓
janitor-cli (CLI / orchestration)
    ↓
janitor-engine (core logic)
    
janitor-broker (Windows service)
    ↓
janitor-engine
```

## What's Complete ✓

- [x] Workspace structure (`janitor-engine`, `janitor-broker`, `janitor-cli` in Cargo.toml)
- [x] Engine crate skeleton — compiles, tests pass
- [x] Core types (`Finding` builder, `ScanContext`, `ScanResult`, `Category`, `RiskLevel`, `TargetKind`)
- [x] Scanner trait (interface every scanner implements)
- [x] Hard blacklist module (safety enforcer)
- [x] Error types
- [x] **[AMENDED 2026-04-27 v2]:** 5 scanners (`temp_dirs`, `recycle_bin`, `browser_cache`, `crash_dumps`, `windows_update`), `janitor-cli` with JSON/HTML export + filtering, `launch.bat` menu, 35+ tests

## What's Next [TODO]

**Phase 1 (MVP — read-only scan + report):**

> **[AMENDED 2026-04-27]:** Items 1, 2, and 4 are done (JSON CLI output). Remaining Phase 1 gap: **HTML report renderer** (item 3) if still desired.

1. `janitor-engine/src/scanners/mod.rs` + `temp_dirs.rs` — first scanner
2. `janitor-cli/src/main.rs` — CLI binary, invokes engine, prints JSON
3. JSON + HTML report renderer
4. Recycle bin + browser cache scanners

**Phase 2:**
- Rule engine (`rules/`)
- SQLite persistence
- Quarantine module
- Privilege broker (named pipe, Windows service)
- Tauri UI
