<!-- PRESERVATION RULE: Never delete or replace content. Append or annotate only. -->

# STYLE_GUIDE (Janitor / Rust)

_Last updated: 2026-04-27_

Project-specific conventions; align with [Rust API guidelines](https://rust-lang.github.io/api-guidelines/) where not contradicted below.

## Naming

- **Rust:** `snake_case` modules/functions, `PascalCase` types, `SCREAMING_SNAKE` constants.
- **Trace:** Link non-obvious safety or policy code to docs: `// [TRACE: DOCS/ARCHITECTURE.md]` (or relevant file).

## Limits (targets)

- ~100 characters per line where practical.
- Prefer functions under ~50 lines; split when scanners or CLI handlers grow.

## Comments

- **WHY** over **WHAT** for non-obvious logic.
- Prefixes: `TODO:`, `FIXME:`, `NOTE:`.

## Errors

- Use `thiserror` for library errors; do not swallow I/O or scan failures — log or propagate via `ScanResult` / `Result` as appropriate.

## Async / Windows

- Engine scanners are sync today; `tokio` is available for future async boundaries. Do not block the broker/UI thread once those layers exist.

## Testing

- Unit tests colocated in `#[cfg(test)]` modules; prefer table-driven cases for path/blacklist rules.
