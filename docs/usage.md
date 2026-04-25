# Usage

The scan flow executes via:

```sh
pure scan --all                       # Scan all categories (default set)
pure scan --type python ~/Desktop     # Scan only python targets
pure scan --type rust --verbose .     # Show item-level paths and sizes
pure scan --list ~/Desktop            # Fast target listing without size calculation
pure sc --current                     # Alias; scan only current directory
```

The delete flow executes via:

```sh
pure run ~/Desktop                    # Interactive category selection + confirmation
pure run --type nodejs -y ~/Desktop   # Non-interactive deletion for one category
pure run --all -y ~/Desktop           # Delete all categories without prompts
pure rn --current --type rust -y      # Alias; current-directory scoped cleanup
```

Category behavior:

- Default categories: xcode, python, rust, nodejs, brew, docker
- Current-directory mode (`--current`) excludes brew and docker categories
- Docker cleanup runs only when docker is requested and `--current` is not used

Help displays via:

```sh
pure --help
pure scan --help
pure run --help
```