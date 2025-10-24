# pure

`pure` is a macOS-focused command line cleaner designed for developers and power users. It scans
common cache locations, log directories, Homebrew artifacts, and other disposable files before
you commit to deletion. Safety comes first: every run starts with a dry-run scan, and nothing is
removed without an explicit confirmation or the `-y/--yes` flag.

## Features

- **Dry-run by default** – `pure scan` reports what can be removed and how much space each
  category represents.
- **Category-aware cleaning** – Target development caches, system caches, logs, Homebrew
  artefacts, browser caches, or the trash individually or all at once.
- **Interactive deletion** – `pure run` without flags lets you choose categories to delete
  interactively before the final confirmation prompt.
- **Persistent exclusions** – Configure glob-style exclusions (e.g. `~/projects/app/.venv`) via
  `pure config --add-exclude` so trusted paths are never touched.
- **macOS aware** – Looks in standard macOS locations such as `~/Library/Caches`, Safari and
  Chrome cache folders, and the user trash.

## Installation

```bash
cargo install --path .
```

The release binary will be available at `target/release/pure`.

## Usage

```bash
pure scan --all              # scan every category (default behaviour)
pure scan --type dev -v      # detailed list of development caches within the current project
pure run                     # scan, pick categories interactively, then confirm before deleting
pure run --type logs -y      # delete log files without prompting for confirmation
pure config --add-exclude ~/projects/app/.venv  # persistently ignore a virtual environment
pure config --path           # show where the configuration file is stored
```

### Categories

| Category  | Description (examples) |
|-----------|------------------------|
| `dev`     | Development caches such as `__pycache__`, `.pytest_cache`, `.ruff_cache`, `node_modules`, `target`, `.venv`, and `DerivedData`. |
| `system`  | macOS system caches located under `~/Library/Caches` and `/Library/Caches`. |
| `logs`    | Log directories including `~/Library/Logs` and `/var/log`. |
| `brew`    | Homebrew caches and build artefacts. |
| `browser` | Safari, Chrome, and Firefox cache directories. |
| `trash`   | The user trash at `~/.Trash`. |

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
