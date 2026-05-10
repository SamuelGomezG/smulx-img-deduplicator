<div align="center">

# smulx-img-deduplicator

**Find, review, and delete duplicate or visually similar images from the terminal.**

[![build](https://github.com/tu-usuario/smulx-img-deduplicator/actions/workflows/ci.yml/badge.svg)](https://github.com/tu-usuario/smulx-img-deduplicator/actions)
[![license: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](./LICENSE)
[![rust: 2024 edition](https://img.shields.io/badge/rust-2024%20edition-orange.svg)](https://www.rust-lang.org)

</div>

```
┌─ Groups ───────────────────┐ ┌─ Files — hash a3f0c1d2e4b56789 ─────────────────────────┐
│ ▶ Group 1  (3 files)       │ │ ▶ [ ] beach_vacation.jpg           2.4 MB   d=0           │
│   Group 2  (2 files)       │ │   [x] beach_vacation_copy.jpg      2.4 MB   d=0           │
│   Group 3  (4 files)       │ │   [ ] beach_vacation_edit.jpg      1.9 MB   d=3           │
│   Group 4  (2 files)       │ │                                                              │
└────────────────────────────┘ └──────────────────────────────────────────────────────────────┘
  4 groups | 1 file marked | Tab: focus  Space: mark  Enter: delete  q: quit
```

---

## What is it?

`smulx-img-deduplicator` is a CLI/TUI tool written entirely in Rust that scans image directories, groups visually similar ones using **perceptual hashing (dHash)**, and provides an interactive terminal interface to review each group and decide what to delete.

Unlike byte-to-byte comparators, it detects as duplicates images that have been **resized, recompressed, cropped, or had brightness or contrast adjustments applied**.

---

## Features

- **High performance** — processes thousands of images in seconds using all available cores (Rayon).
- **Visual similarity** — perceptual hashing with transitive clustering via Union-Find: if A ≈ B and B ≈ C, all three end up in the same group even if A and C are not directly similar.
- **Safe by default** — moves files to the operating system trash instead of permanently deleting them. Never allows deleting the last copy in a group.
- **Interactive** — two-panel TUI for navigating groups, marking files, and confirming deletions without leaving the terminal.
- **No external dependencies** — a single static binary, no runtime, no Python, no C++.

---

## Supported formats

`JPEG` · `PNG` · `WebP` · `GIF` · `TIFF` · `BMP`

> RAW formats (CR2, NEF, ARW) are not supported in this version.

---

## Installation

**Requirement:** [Rust stable](https://rustup.rs/) (2024 edition).

### From source

```bash
git clone https://github.com/tu-usuario/smulx-img-deduplicator
cd smulx-img-deduplicator
make install
```

Installs the binary at `~/.local/bin/smulx-dedup`. To change the destination:

```bash
make install INSTALL_DIR=/usr/local/bin
```

### With Cargo

```bash
cargo install --path .
```

---

## Usage

```
smulx-dedup <DIRECTORY>... [OPTIONS]
```

### Examples

```bash
# Scan ~/Pictures with the recommended threshold (5)
smulx-dedup ~/Pictures

# Scan multiple directories at once
smulx-dedup ~/Pictures ~/Downloads/photos /mnt/backup

# Exact duplicates only
smulx-dedup ~/Pictures --threshold 0

# More aggressive detection: crops, filters, watermarks
smulx-dedup ~/Pictures --threshold 10

# Permanent deletion instead of trash
smulx-dedup ~/Pictures --use-trash false

# Export groups to JSON before opening the TUI
smulx-dedup ~/Pictures --export-json groups.json
```

### Options reference

| Option | Default | Description |
|---|---|---|
| `--threshold <N>` | `5` | Similarity threshold in Hamming distance. `0` = exact only. Recommended range: 3–8. |
| `--use-trash` | `true` | Sends deleted files to the system trash. |
| `--export-json <PATH>` | — | Exports the group list to JSON before opening the TUI. |
| `--log-level <LEVEL>` | `warn` | Log verbosity: `error` `warn` `info` `debug` `trace`. Writes to stderr. |

### Keyboard shortcuts

| Key | Action |
|---|---|
| `↑` `↓` · `k` `j` | Navigate the focused list |
| `Tab` | Toggle focus between the groups panel and the files panel |
| `Space` | Mark / unmark file for deletion |
| `Enter` · `x` | Delete marked files in the current group (prompts for confirmation) |
| `v` | Open the selected file with the system default viewer |
| `q` · `Esc` | Quit without deleting anything |

---

## About the similarity threshold

The threshold controls how different two images can be and still be considered similar. It is measured in **Hamming distance**: the number of differing bits between two 64-bit perceptual hashes.

| Threshold | Detects |
|---|---|
| `0` | Exact duplicates only (bit-for-bit identical content) |
| `3`–`5` | Resizing, recompression, slight brightness or contrast adjustments |
| `6`–`10` | Crops, filters, watermarks, lossy format conversions |

If you don't know where to start, **`--threshold 5`** is the recommended starting point.

---

## Development

### Requirements

- Rust stable (2024 edition) — install with [rustup](https://rustup.rs/)
- `make`

### Getting started

```bash
git clone https://github.com/tu-usuario/smulx-img-deduplicator
cd smulx-img-deduplicator

# Full pipeline: format + lint + tests
make check

# Run against a local gallery
make dev GALLERY_PATH=~/Pictures

# Tests only
make test
```

### Makefile targets

| Target | Description |
|---|---|
| `make check` | `fmt` + `lint` + `test` in a single invocation. Run before every commit. |
| `make fmt` | Applies `rustfmt` to the source code. |
| `make lint` | Runs `clippy` with warnings as errors. |
| `make test` | Runs unit tests and integration tests. |
| `make build` | Compiles the optimized binary (`--release`). |
| `make install` | Compiles and installs to `~/.local/bin/` (configurable with `INSTALL_DIR`). |
| `make dev` | Runs the project with development arguments (configurable with `GALLERY_PATH`). |
| `make clean` | Removes build artifacts. |

### Project structure

```
src/
├── main.rs          # Entry point and phase orchestration
├── cli.rs           # Command-line arguments (clap)
├── scanner.rs       # File discovery (jwalk)
├── hasher.rs        # Parallel perceptual hashing (rayon + img_hash)
├── bktree.rs        # BK-Tree for similarity search
├── cluster.rs       # Connected components clustering (Union-Find)
├── error.rs         # Error types (thiserror)
└── tui/
    ├── app.rs       # Application state
    ├── ui.rs        # Rendering (ratatui)
    └── events.rs    # Event loop (crossterm)
tests/
├── integration_scanner.rs    # File discovery on temporary dirs
└── integration_pipeline.rs   # Full pipeline with synthetic images
```

### Tests

The project follows **strict TDD**: tests are written before production code. Each business logic module has its internal `#[cfg(test)]` block; cross-module or filesystem-touching tests live in `tests/`.

```bash
# All tests
cargo test --all

# A single integration test file
cargo test --test integration_pipeline

# With log output
SMULX_LOG=debug cargo test --all -- --nocapture
```

---

## Contributing

Contributions are welcome. Please:

1. Open an issue describing the bug or proposal before submitting a PR.
2. Make sure `make check` passes without errors before committing.
3. Follow the TDD cycle: tests first, implementation after.

---

## License

`smulx-img-deduplicator` is available under the [MIT](./LICENSE) license.
