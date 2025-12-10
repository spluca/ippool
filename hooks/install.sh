#!/bin/bash
#
# Install git hooks for the ippool project
#

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
GIT_HOOKS_DIR="$(git rev-parse --git-dir)/hooks"

echo "Installing git hooks..."

# Install pre-commit hook
if [ -f "$SCRIPT_DIR/pre-commit" ]; then
    ln -sf "../../hooks/pre-commit" "$GIT_HOOKS_DIR/pre-commit"
    chmod +x "$SCRIPT_DIR/pre-commit"
    echo "✓ Installed pre-commit hook"
else
    echo "✗ pre-commit hook not found"
    exit 1
fi

echo ""
echo "Git hooks installed successfully!"
echo "The pre-commit hook will run 'cargo fmt --check' before each commit."
