<!-- PRESERVATION RULE: Never delete or replace content. Append or annotate only. -->

# Audit log — 2026-04-27

Structured pass over build, static analysis, dependency surface, and doc drift.

## Commands run

| Command | Result |
|---------|--------|
| `cargo test --workspace` | **Pass** — 64 tests in `janitor-engine`; 0 in `janitor-cli`, `janitor-broker`, `janitor-ui` binaries |
| `cargo clippy --workspace --all-targets` | **Pass** with warnings → fixed `clippy::print_literal` in `janitor-cli` (see commit) |
| `cargo audit` | **Not run** — `cargo-audit` not installed in this environment. **Recommendation:** `cargo install cargo-audit` then `cargo audit` before releases. |

## Workspace shape

| Crate | Role | Notes |
|-------|------|------|
| `janitor-engine` | Lib | Blacklist + 5 scanners (`temp_dirs`, `recycle_bin`, `browser_cache`, `crash_dumps`, `windows_update`) |
| `janitor-cli` | Bin `janitor` | Scan / list / about / JSON+HTML export |
| `janitor-broker` | Bin stub | Phase 2 privilege broker |
| `janitor-ui` | Bin `janitor-ui` | Tauri 2 + WebView2; pulls large transitive graph |

## Security / safety (code review snapshot)

- **Blacklist:** Still the primary gate for path safety; traversal tests present.
- **Phase 1:** Read-only scan posture; no deletion paths in engine from this review.
- **UI:** Tauri/WebView2 expands attack surface vs CLI-only; future IPC/plugin boundaries should stay documented in `ARCHITECTURE.md` as they land.
- **Secrets:** No API keys observed in tree; keep `.env` gitignored (already in `.gitignore`).

## Documentation drift corrected

- `DOCS/SUMMARY.md` previously omitted `janitor-ui` and understated test count — updated same day.
- `README.md` Testing section: test count line aligned with `cargo test` output (64 engine tests).

## Follow-ups

1. Add CI job: `cargo clippy -- -D warnings`, `cargo test --workspace`, optional `cargo audit`.
2. Install and run `cargo audit` locally or in CI.
3. Consider `deny.toml` / `cargo-deny` for license + advisory policy on Tauri-heavy lockfile.
