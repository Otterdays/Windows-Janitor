# Janitor — Safe Windows PC Cleaner

Safe, transparent Windows system cleaner written in Rust. **Phase 1 is read-only** — scans your system for junk files but doesn't modify anything.

## Quick Start

**Desktop UI (recommended):**
```bash
cargo build --release -p janitor-ui
target\release\janitor-ui.exe
# Requires WebView2 (pre-installed on Windows 11)
```

**Interactive menu:**
```bash
launch.bat
```

**Command line:**
```bash
cargo run -p janitor-cli -- scan              # Human-readable report
cargo run -p janitor-cli -- scan --json       # JSON output
cargo run -p janitor-cli -- scan --html out.html  # HTML report
cargo run -p janitor-cli -- list              # Show all scanners
```

## Features

✓ **5 Scanners:**
- Temporary files (`%TEMP%`, `%LOCALAPPDATA%\Temp`, `C:\Windows\Temp`)
- Recycle Bin (`C:\$Recycle.Bin`)
- Browser cache (Chrome, Edge, Firefox, Brave, Opera)
- Crash dumps (`.dmp`, `.log` files)
- Windows Update leftovers

✓ **Safety:**
- Hard blacklist prevents touching System32, WinSxS, pagefile.sys, etc.
- Path traversal protection
- Confidence scoring for heuristic findings
- **Read-only in Phase 1** — no files are modified

✓ **Reporting:**
- Terminal output (human-readable)
- JSON export
- HTML reports (with color-coded risk levels)
- Filter by: minimum size, risk level, category

## Layout

| Path | Role |
|------|------|
| `janitor-engine/` | Core types, blacklist, 5 scanners, finding models |
| `janitor-cli/` | `janitor` binary — scan, list, about, export to JSON/HTML |
| `janitor-ui/` | Tauri 2 desktop app — dark UI, live filter, IPC to engine |
| `janitor-broker/` | Stub for Phase 2 (privilege broker / Windows service) |

## CLI Usage

```bash
# Quick scan
janitor scan

# Scan with filters
janitor scan --min-size-mb 10 --risk high --category TempFiles

# Export formats
janitor scan --json           # to stdout
janitor scan --output out.json  # to file
janitor scan --html report.html # HTML report

# Single scanner
janitor scan --scanner temp_dirs

# List all scanners
janitor list

# Safety info
janitor about
```

## Testing

```bash
cargo test --workspace    # Engine: 64 unit tests; UI/broker CLI bins: 0
cargo test -p janitor-engine  # Engine only
```

## Documentation

- [`FOUNDATION.md`](FOUNDATION.md) — Implementation details and scanners
- [`ARCHITECTURE.md`](ARCHITECTURE.md) — Layered trust model, security design
- [`PROJECT_LAYOUT.md`](PROJECT_LAYOUT.md) — File structure and roadmap
- [`CHANGELOG.md`](CHANGELOG.md) — Version history

## Roadmap

**Phase 2 (coming):**
- Deletion with quarantine (undo-able, audit log)
- Rule engine (TOML-based, signed)
- SQLite persistence
- Privilege broker (Windows service + IPC)
- Tauri UI

**Phase 2+ scanners:**
- Windows Update cleanup (DISM integration)
- Thumbnail cache
- Prefetch files
- Event logs
- Duplicate detection

## Why Janitor?

- **Transparent** — Every finding shows its source
- **Safe** — Hard blacklist, defense-in-depth
- **Fast** — Parallel scanners, <90 sec for 500GB
- **Open source** — MPL-2.0 licensed

## Safety Philosophy

1. **Reversible by default** (Phase 2: quarantine before delete)
2. **Transparent** (show every rule)
3. **Local-only** (no network in Phase 1)
4. **Open source** (audit the code)
5. **Privilege-minimal** (UI never runs as admin)
6. **Read-before-write** (Phase 1 read-only; Phase 2 validates)
7. **Fail-safe** (errors abort; never partial)
8. **Blacklist enforced last** (defense-in-depth)

## License

Mozilla Public License 2.0 — See `Cargo.toml`.
