<!-- PRESERVATION RULE: Never delete or replace content. Append or annotate only. -->

# SBOM — software bill of materials

_Last updated: 2026-04-27_

Workspace manifests: root `Cargo.toml`, `janitor-engine/Cargo.toml`, `janitor-cli/Cargo.toml`, `janitor-broker/Cargo.toml`. **Resolved versions** for transitive crates live in `Cargo.lock` at repository root.

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

## Notes

- No new crates were added in this documentation pass; SBOM reflects manifests as of 2026-04-27.
- Re-run `cargo tree -p janitor-engine` (and siblings) before release for a full transitive listing.
