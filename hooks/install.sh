#!/bin/bash
#
# Install git hooks for the ippool project
#
# This script creates symlinks from .git/hooks/ to the versioned hooks in hooks/
#

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
GIT_DIR="$(git rev-parse --git-dir)"
GIT_HOOKS_DIR="$GIT_DIR/hooks"

echo "Installing git hooks to $GIT_HOOKS_DIR..."
echo ""

# Install pre-commit hook
if [ -f "$SCRIPT_DIR/pre-commit" ]; then
    # Make the source hook executable
    chmod +x "$SCRIPT_DIR/pre-commit"
    
    # Create symlink from .git/hooks/pre-commit to hooks/pre-commit
    ln -sf "../../hooks/pre-commit" "$GIT_HOOKS_DIR/pre-commit"
    
    echo "✓ Installed pre-commit hook"
    echo "  Source: hooks/pre-commit"
    echo "  Target: .git/hooks/pre-commit -> ../../hooks/pre-commit"
else
    echo "✗ Error: pre-commit hook not found at $SCRIPT_DIR/pre-commit"
    exit 1
fi

echo ""
echo "✓ Git hooks installed successfully!"
echo ""
echo "The pre-commit hook will run 'cargo fmt --check' before each commit."
echo "To bypass temporarily, use: git commit --no-verify"
