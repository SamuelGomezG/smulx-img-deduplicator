# AGENTS.md

## Build & test commands

- **Before every commit:** `make check` (fmt + lint + test)
- **Format only:** `make fmt`
- **Lint only:** `make lint` (clippy with `-D warnings`)
- **Run all tests:** `make test` or `cargo test --all`
- **Build release binary:** `make build` or `cargo build --release`
- **Dev run against gallery:** `make dev GALLERY_PATH=/path/to/images`
- **Install binary locally:** `make install` or `make install INSTALL_DIR=/some/path`

## Architecture

- Rust edition **2024**
- Binary name is `smulx-dedup`, not `smulx-img-deduplicator`
- **All source code is defined in `smulx-img-deduplicator-design.md`** — that is the spec. Module signatures, APIs, and test bodies live there
- `anyhow` for application-layer errors, `thiserror` for library error types (in `src/error.rs`)
- `tracing` logs to **stderr** only — stdout is reserved for the TUI

## Dependency versions

- All crate versions resolve to **latest stable** at install time via `cargo add <crate>` — no pinned versions in the design doc
- The `docs.rs/<crate>/latest/` links below are the **single source of truth** for each crate's API; consult them before implementing any module that uses the corresponding crate

## Crate documentation (source of truth)

| Crate | Role | Docs URL |
|---|---|---|
| `clap` | CLI argument parsing (derive macros) | https://docs.rs/clap/latest/clap/ |
| `jwalk` | Parallel directory traversal | https://docs.rs/jwalk/latest/jwalk/ |
| `image` | Image format decoding and in-memory manipulation | https://docs.rs/image/latest/image/ |
| `img_hash` | Perceptual hashing (dHash / Gradient Hash) | https://docs.rs/img_hash/latest/img_hash/ |
| `rayon` | Parallel iterators and global thread pool | https://docs.rs/rayon/latest/rayon/ |
| `ratatui` | TUI framework: layouts, widgets, rendering | https://docs.rs/ratatui/latest/ratatui/ |
| `crossterm` | Terminal backend: key events, raw mode, alt screen | https://docs.rs/crossterm/latest/crossterm/ |
| `anyhow` | Ergonomic error handling in application code | https://docs.rs/anyhow/latest/anyhow/ |
| `thiserror` | Custom error types with `derive` | https://docs.rs/thiserror/latest/thiserror/ |
| `trash` | Move files to OS trash | https://docs.rs/trash/latest/trash/ |
| `tracing` | Structured instrumentation and logging | https://docs.rs/tracing/latest/tracing/ |
| `tracing-subscriber` | Trace collector config (filtering, formatting) | https://docs.rs/tracing_subscriber/latest/tracing_subscriber/ |
| `serde` | Generic serialization / deserialization | https://docs.rs/serde/latest/serde/ |
| `serde_json` | JSON serialization / deserialization | https://docs.rs/serde_json/latest/serde_json/ |
| `tempfile` | Temp dirs/files for tests (dev-dependency) | https://docs.rs/tempfile/latest/tempfile/ |

> **`ratatui` note:** Prefer the introduction guide at https://ratatui.rs/ before the API reference — it explains the mental model (terminal setup, event loop, buffer rendering).
>
> **`img_hash` note:** Correct API is `HasherConfig::new().hash_size(8, 8).hash_alg(HashAlg::Gradient).to_hasher()` then `hasher.hash_image(&img)`. Result is `ImageHash` → `u64` via `hash.as_bytes()` (8 bytes, little-endian). Distance via `hash1.dist(&hash2)` returning `u32`.

## TDD rules (strict)

1. Tests first, implementation after. No code without a corresponding test.
2. Unit tests live in `#[cfg(test)]` blocks inside each source file
3. Integration tests live in `tests/` (file discovery, end-to-end pipeline)
4. No commit with failing tests
5. Pure logic functions (`compute_hash`, `hamming_distance`, BK-tree insert/search, `build_clusters`) must be `pub(crate)` or extracted to standalone functions for isolated testing
6. All test images are **synthetic** (generated via the `image` crate; no external JPEG/PNG fixtures)
7. Integration tests use `tempfile::TempDir` for filesystem isolation

## Key conventions

- Keep functions pure where possible; I/O and side effects are at the boundaries (scanner, TUI event loop, deletion)
- The TUI render functions (`ui.rs`) have no unit tests — verified visually during development
- `make check` is the CI/pre-commit gate: `fmt` → `lint` → `test`
- The design doc is gitignored — use it as reference, not as a tracked file
