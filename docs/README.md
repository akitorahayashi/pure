# Documentation

This directory stores repository-level documentation for development operations and CI/CD behavior.

Current operational contract:
- Local setup is driven by `just setup`.
- Verification is driven by `just check` and `just test`.
- Coverage is collected by `just coverage`.
- CI workflows delegate static checks, tests, coverage, and build to reusable workflows.
- Reusable workflows invoke repository recipes through `mise exec -- just <recipe>`.