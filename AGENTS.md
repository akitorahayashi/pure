# pure Development Notes

## Project Summary
`pure` is a Rust CLI that scans and cleans macOS caches. The binary exposes three primary
subcommands:
- `scan`: dry-run discovery of reclaimable disk space per category.
- `run`: deletion workflow (interactive by default, supports `--type`, `--all`, `-y`).
- `config`: manage the TOML configuration file with persistent exclusion globs.

## Key Modules
- `src/model.rs` – Category definitions and scan/deletion report structures.
- `src/scanners/` – Modular language-specific scanners (xcode, python, rust, nodejs, brew).
- `src/commands/scan.rs` / `run.rs` / `config_cmd.rs` – User-facing command implementations.
- `src/config.rs` – Config file loading/saving and glob compilation.
- `src/path.rs` – Path utilities and resolution helpers.
- `src/format.rs` – Byte formatting utilities.

## Coding Guidelines
- Keep output human-friendly: use `format::format_bytes`/`path::display_path` for user-facing size/path
  strings.
- Respect exclusions in both scan and deletion flows. All scanners handle exclusions via globset.
- Language-specific scanners implement the `CategoryScanner` trait with parallel execution support.
- Desktop-focused safety: defaults to ~/Desktop scanning to avoid system areas.
- Prefer small, testable helpers. Unit tests can live alongside modules, while high-level CLI
  flows belong in `tests/`.
- Avoid deleting files that were not surfaced by the scan report.

## Testing & Tooling
- `cargo fmt --check`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `RUST_TEST_THREADS=1 cargo test --all-targets --all-features`

Integration tests in `tests/` configure `HOME` and `XDG_CONFIG_HOME` to temporary directories to
keep the host environment untouched.
