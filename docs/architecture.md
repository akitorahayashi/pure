# Architecture

## Canonical Model

- Category: Cleanup domain unit (`xcode`, `python`, `rust`, `nodejs`, `brew`, `docker`).
- Scan Item: A concrete file or directory candidate with measured size.
- Scan Report: Category-grouped aggregation of reclaimable targets.
- Run Plan: User-selected subset of scan results approved for deletion.

## Ownership Boundaries

| Boundary | Path | Responsibility |
|---|---|---|
| Interface adapter | `src/main.rs` | CLI parsing, argument shaping, and command dispatch |
| Command orchestration | `src/commands/` | Scan and run flow control, reporting, and confirmation |
| Scanner owner | `src/scanners/` | Category-specific target discovery |
| Docker integration | `src/docker_cleanup.rs` | Docker target listing, scan, and prune execution |
| Domain model | `src/model.rs` | Category, scan item, and scan report model |
| Filesystem safety | `src/path.rs` | Path resolution, display shaping, and safe deletion helpers |
| Output formatting | `src/format.rs` | Human-readable byte formatting |
| Error kernel | `src/error.rs` | Typed application error model |

## Package Structure

```text
src/
├── main.rs
├── lib.rs
├── error.rs
├── format.rs
├── model.rs
├── path.rs
├── docker_cleanup.rs
├── commands/
│   ├── mod.rs
│   ├── scan.rs
│   └── run.rs
└── scanners/
    ├── mod.rs
    ├── xcode.rs
    ├── python.rs
    ├── rust.rs
    ├── nodejs.rs
    ├── brew.rs
    └── generic.rs

tests/
├── scan.rs
├── run.rs
└── aliases.rs
```

## Execution Model

- `scan` performs discovery first and size calculation second, with parallel execution for throughput.
- `run` always starts from a scan report, then applies selection, confirmation, and deletion phases.
- Docker cleanup is handled as a dedicated path and joined with filesystem deletion when enabled.

## Safety Invariants

- Scanning is non-destructive.
- Deletion requires explicit confirmation unless `-y/--yes` is provided.
- Current-directory mode prevents system-wide categories (brew and docker) from running.