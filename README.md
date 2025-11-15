## Overview

`pure` is a desktop-focused command line cleaner designed for developers. It scans development
caches and build artifacts in your desktop projects to help reclaim disk space. Safety comes
first: every run starts with a dry-run scan, and nothing is removed without an explicit
confirmation or the `-y/--yes` flag.

## Features

- **Dry-run by default** – `pure scan` reports what can be removed and how much space each
  category represents.
- **Language-aware cleaning** – Target Python, NodeJS, Rust, Xcode, or Homebrew caches
  individually or all at once.
- **Fast preview** – `pure scan --list` quickly shows what cleanup targets exist without
  calculating sizes.
- **Interactive deletion** – `pure run` without flags lets you choose categories to delete
  interactively before the final confirmation prompt.
- **Desktop-focused** – Defaults to scanning `~/Desktop` for safety, avoiding system areas.
- **Current directory mode** – Use `--current` to scan only the current directory; automatically skips system-wide categories like Homebrew.

## Installation

```bash
cargo install --path .
```

The release binary will be available at `target/release/pure`.

## Usage

```bash
pure scan --all                           # scan every category (defaults to ~/Desktop)
pure scan --current                       # scan only the current directory instead of ~/Desktop
pure scan --list                          # quickly list cleanup targets without calculating sizes
pure scan --type python -v                # detailed list of Python caches
pure scan --type nodejs                   # scan NodeJS projects only
pure run                                  # scan, pick categories interactively, then confirm before deleting
pure run --current --type rust -y         # delete Rust build artifacts in current directory without prompting
pure config --path                        # show where the configuration file is stored
```

### Categories

| Category  | Description (examples) |
|-----------|------------------------|
| `xcode`   | Smart Xcode/Swift cleanup. Detects project-local `DerivedData` and, only when a `Package.swift` is present, also cleans sibling `.build`, `.swiftpm`, and `Package.resolved`. When not using `--current`, also scans vetted global caches such as `~/Library/Developer/Xcode/DerivedData`, `~/Library/Caches/com.apple.dt.Xcode`, `~/Library/Developer/CoreSimulator/Caches`, and SwiftPM caches under `~/Library/Caches/org.swift.swiftpm`. |
| `python`  | Python caches such as `__pycache__`, `.pytest_cache`, `.ruff_cache`, `.mypy_cache`, `.venv`, and `.uv-cache`. |
| `rust`    | Rust build artifacts in `target` directories. |
| `nodejs`  | NodeJS development artifacts including `node_modules`, `.next`, `.nuxt`, and `.svelte-kit`. |
| `brew`    | Homebrew caches and build artifacts. Note: Skipped when using `--current` option. |
| `docker`  | Docker cache and unused data (`image prune -a`, `container prune`, `volume prune`, `network prune`, `builder prune -a`). Skipped when using the `--current` option. |

### Safety Model

1. **Transparency** – Scans always show the total reclaimable size per category. Use
   `--verbose` for individual items.
2. **Control** – Restrict actions with `--type <category>`, `--all`, or the interactive
   prompt in `pure run`.
3. **Confirmation** – Destructive actions require confirmation unless `-y/--yes` is supplied.
4. **Exclusions** – Add permanent exclude globs to `~/.config/pure/config.toml` (or the
   platform-specific config directory). Exclusions are respected during scanning and deletion.

### Configuration File

`pure` stores configuration in `~/.config/pure/config.toml` (or the directory pointed to by
`$XDG_CONFIG_HOME`). The structure is intentionally simple:

```toml
exclude = [
  "~/projects/app/.venv",
  "~/experiments/big-dataset"
]
```

Use `pure config --edit` to open the file in your `$EDITOR` or `$VISUAL` editor. The configuration
file is created on demand if it does not already exist.

## Development

Useful commands when working on the project:

- `cargo fmt --check`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `RUST_TEST_THREADS=1 cargo test --all-targets --all-features`

The integration tests rely on temporary directories and set `HOME`/`XDG_CONFIG_HOME` to avoid
mutating your real environment.
