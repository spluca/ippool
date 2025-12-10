# Git Hooks

## Overview

Git hooks are scripts that run automatically at specific points in the Git workflow. This project includes hooks to ensure code quality.

## Installation

From the project root, run:

```bash
./hooks/install.sh
```

This creates symlinks from `.git/hooks/` to the versioned hooks in `hooks/`:
- `.git/hooks/pre-commit` → `../../hooks/pre-commit`

## Pre-commit Hook

Runs `cargo fmt --check` before each commit to ensure code is properly formatted.

### How it works

1. **On commit:** Git automatically executes `.git/hooks/pre-commit`
2. **Check format:** The hook runs `cargo fmt --check`
3. **Success:** If formatted correctly, commit proceeds
4. **Failure:** If not formatted, commit is blocked with instructions

### Example output

**Success:**
```
Running cargo fmt check...
✓ Code formatting check passed!
```

**Failure:**
```
Running cargo fmt check...
Error: Code is not formatted properly.
Please run 'cargo fmt' to format your code before committing.

To format all code, run:
  cargo fmt

To bypass this check (not recommended), use:
  git commit --no-verify
```

### Fix formatting issues

```bash
cargo fmt
```

### Bypass (not recommended)

To skip the hook temporarily:

```bash
git commit --no-verify
```

## Requirements

- Rust and Cargo installed
- `rustfmt` component (auto-installed if missing)

## Why symlinks?

Using symlinks allows the hooks to be versioned in git while still being active in `.git/hooks/`. This way:
- Hooks are part of the repository
- Updates to hooks are automatically pulled
- Each developer runs `./hooks/install.sh` once after cloning
