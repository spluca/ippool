# Git Hooks

## Pre-commit Hook

Executes `cargo fmt --check` before each commit to ensure code is properly formatted.

### Installation

From the project root, run:

```bash
ln -sf ../../hooks/pre-commit .git/hooks/pre-commit
chmod +x .git/hooks/pre-commit
```

Or simply run:

```bash
./hooks/install.sh
```

### Usage

The hook runs automatically on `git commit`. If code is not formatted:

```
Error: Code is not formatted properly.
Please run 'cargo fmt' to format your code before committing.
```

To fix, run:

```bash
cargo fmt
```

### Bypass (not recommended)

To skip the hook temporarily:

```bash
git commit --no-verify
```

### Requirements

- Rust and Cargo installed
- `rustfmt` component (auto-installed if missing)
