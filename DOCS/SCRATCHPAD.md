<!-- PRESERVATION RULE: Never delete or replace content. Append or annotate only. -->

# SCRATCHPAD

## Active tasks

- (none — docs + git prep completed 2026-04-27)

## Blockers

- (none)

## Last actions (newest first)

1. **2026-04-27:** Committed + pushed workspace + audit docs — `5423abd` (`chore: workspace sync, audit DOCS, fix cli clippy warnings`); SCRATCHPAD touch `9f3617b`.
2. **2026-04-27:** Project audit — `cargo test --workspace`, `clippy --workspace`; `DOCS/debugs/debug_2026-04-27_audit.md`; SUMMARY/SBOM/CHANGELOGs + README test count; fixed `janitor-cli` `clippy::print_literal`.
3. **2026-04-27:** Pushed `master` to [Otterdays/Windows-Janitor](https://github.com/Otterdays/Windows-Janitor); workspace `repository` URL updated in `Cargo.toml` (`1fae396`).
4. **2026-04-27:** Appended `PROJECT_LAYOUT.md` amendment (tree vs reality; What's Complete).
5. **2026-04-27:** Added `DOCS/` workflow files, root `README.md`, `.gitignore`; amended `CHANGELOG.md` / `PROJECT_LAYOUT.md` for Phase 1 accuracy; `git init`.
6. **2026-04-27:** Verified `cargo test` — engine tests (later **64** with `crash_dumps` / `windows_update`).
7. **2026-04-27:** Initial git commit `57f3561` on `master` (working tree clean).

## Next steps (suggested)

- **[DONE 2026-04-27]:** `Cargo.toml` `repository` → `https://github.com/Otterdays/Windows-Janitor`
- Install **`cargo-audit`** (or CI) and run **`cargo audit`** on each release.
- Phase 2: broker IPC, rules, persistence, hardened UI IPC (per `ARCHITECTURE.md`).

## Out-of-Scope Observations

- Folder name `PC-Scan-1` vs crate branding `janitor` — intentional rename TBD by maintainer.
