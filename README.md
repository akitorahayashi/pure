# pure

Desktop-focused command line cleaner.

`pure` scans development caches and build artifacts in desktop projects to help reclaim disk
space. Scans are dry-run by default, and deletion requires explicit confirmation unless `-y/--yes`
is supplied.

## Quick Start

### Installation

```bash
cargo install --path .
```

The release binary will be available at `target/release/pure`.

### Verification

```bash
pure --version
pure scan --list
```

### Common Commands

```bash
pure scan --all                  # Scan every category
pure sc --current                # Alias for scan in current-directory mode
pure run                         # Scan, select categories, and confirm deletion
pure rn --current --type rust -y # Alias for run with explicit deletion
pure scan --type python -v       # Show detailed Python cleanup targets
```

### Categories

| Category  | Description |
|-----------|-------------|
| `xcode`   | Project-local Xcode/Swift caches and, outside `--current`, vetted global Xcode and SwiftPM caches. |
| `python`  | Python caches such as `__pycache__`, `.pytest_cache`, `.ruff_cache`, `.mypy_cache`, `.venv`, and `.uv-cache`. |
| `rust`    | Rust build artifacts in `target` directories. |
| `nodejs`  | NodeJS artifacts including `node_modules`, `.next`, `.nuxt`, and `.svelte-kit`. |
| `brew`    | Homebrew caches and build artifacts. Skipped in `--current` mode. |
| `docker`  | Docker cache and unused data. Skipped in `--current` mode. |

### Safety Model

1. Scans report reclaimable size per category.
2. `--type <category>`, `--all`, and interactive selection constrain deletion scope.
3. Destructive actions require confirmation unless `-y/--yes` is supplied.
4. Exclusions are respected during scanning and deletion.

## Documentation

- [Docs](docs/README.md): Usage, architecture, configuration, and testing references.
- [Contributing](CONTRIBUTING.md): Development guidelines and verification commands.
