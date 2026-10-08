#!/usr/bin/env bash
# Ecosystem report prototype — read-only research/reporting.
# Produces a structured report of Rust ecosystem signals.
# Does NOT modify any files. Does NOT create issues.
# Usage: bash scripts/ecosystem-report.sh [daily|weekly|monthly]
set -uo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$REPO_ROOT"

MODE="${1:-daily}"

echo "=== Rust Ecosystem Report ==="
echo "Mode: $MODE"
echo "Date: $(date -u +%Y-%m-%d)"
echo "Repo: $REPO_ROOT"
echo

# --- Official sources (placeholders for future adapters) ---
echo "--- Official Sources ---"
echo "Rust Blog: https://blog.rust-lang.org/"
echo "Inside Rust: https://blog.rust-lang.org/inside-rust/"
echo "This Week in Rust: https://this-week-in-rust.org/"
echo "Rust Foundation: https://foundation.rust-lang.org/"
echo

# --- Community sources (placeholders) ---
echo "--- Community Sources ---"
echo "Reddit r/rust: https://reddit.com/r/rust"
echo "Hacker News: https://news.ycombinator.com/"
echo

# --- Potential skill impacts ---
echo "--- Potential Skill Impacts ---"
echo "(No automated impact analysis yet. Manual review required.)"
echo

# --- Repository health ---
echo "--- Repository Health ---"
SKILL_COUNT=$(find skills -name SKILL.md -type f | grep -c .)
REF_COUNT=$(find references -name '*.md' -type f | grep -c .)
EXAMPLE_COUNT=$(find examples -name Cargo.toml -type f | grep -c .)
echo "Skills: $SKILL_COUNT"
echo "References: $REF_COUNT"
echo "Examples: $EXAMPLE_COUNT"
echo

# --- Stale version check (basic) ---
echo "--- Version References ---"
echo "Checking for version claims in skills..."
grep -r "Rust 1\." skills/ --include="*.md" | head -5
echo

echo "=== End of Report ==="
echo "This is a read-only report. No files were modified."
echo "For automated issue generation, see docs/issue-automation.md."
