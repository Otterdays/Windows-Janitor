<!-- PRESERVATION RULE: Never delete or replace content. Append or annotate only. -->

# SBOM — software bill of materials

_Last updated: 2026-04-27 (audit pass)_

Workspace manifests: root `Cargo.toml`, `janitor-engine/Cargo.toml`, `janitor-cli/Cargo.toml`, `janitor-broker/Cargo.toml`, `janitor-ui/Cargo.toml`. **Resolved versions** for transitive crates live in `Cargo.lock` at repository root.

**Advisories:** Run `cargo audit` (install with `cargo install cargo-audit` if needed) before release; not executed in the 2026-04-27 doc audit (tool absent).

## Workspace-level dependencies (`[workspace.dependencies]`)

| Package | Version constraint (manifest) |
|---------|-----------------------------|
| thiserror | 1.0 |
| serde | 1.0 (derive) |
| serde_json | 1.0 |
| tokio | 1 (full) |
| tracing | 0.1 |
| tracing-subscriber | 0.3 (env-filter) |

## `janitor-engine`

| Package | Version constraint |
|---------|-------------------|
| (inherits workspace deps above) | |
| windows | 0.52 (Win32 features) |
| blake3 | 1.5 |
| rayon | 1.8 |
| walkdir | 2.4 |
| chrono | 0.4 |
| uuid | 1.6 (v4, serde) |

**Dev:** tempfile 3.8

## `janitor-cli`

| Package | Version constraint |
|---------|-------------------|
| janitor-engine | path `../janitor-engine` |
| serde_json | workspace |
| clap | 4.4 (derive) |
| rayon | 1.8 |
| chrono | 0.4 |

## `janitor-broker`

| Package | Version constraint |
|---------|-------------------|
| janitor-engine | path `../janitor-engine` |

## `janitor-ui`

| Package | Version constraint |
|---------|-------------------|
| janitor-engine | path `../janitor-engine` |
| tauri | 2 |
| tauri-build | 2 (build-dependencies) |
| serde | 1 (derive) |
| serde_json | 1 |
| rayon | 1.8 |

**Features:** `default = ["custom-protocol"]`; `custom-protocol` enables `tauri/custom-protocol`.

> **NOTE:** Tauri pulls a large dependency tree (WebView2, wry, etc.). Security review should include lockfile advisories, not only direct deps.

## Notes

- Manifest-level table above; lockfile pins exact semver (e.g. `tauri` resolved to 2.x patch in `Cargo.lock`).
- Re-run `cargo tree -p janitor-ui` / `janitor-engine` before release for transitive detail.
