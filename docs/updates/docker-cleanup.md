# Docker Cleanup

- Added a dedicated Docker category with scan/list/run handling that is skipped with `--current`.
- Implemented `docker_cleanup` helpers to summarize `docker system df` output and invoke prune commands.
- Extended pruning to stopped containers and unused networks alongside existing image/volume/builder cleanups.
- Documented and tested the new behavior, including CLI list output expectations.
