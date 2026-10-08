#!/usr/bin/env bash
# rustoria installation helper.
# Installs skills into an agent's skills directory.
# Usage: scripts/setup.sh [target-dir]
#   Default target: ~/.claude/skills (or $SKILLS_DIR if set)
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$REPO_ROOT"

TARGET="${1:-${SKILLS_DIR:-$HOME/.claude/skills}}"

echo "=== rustoria setup ==="
echo "Repo:    $REPO_ROOT"
echo "Target:  $TARGET"
echo

if [ ! -d "$REPO_ROOT/skills" ]; then
  echo "[FAIL] skills/ directory not found"
  exit 1
fi

mkdir -p "$TARGET"

# Every supported agent discovers skills as a directory bundle:
#   TARGET/<skill-name>/SKILL.md
# (Agent Skills open standard, https://agentskills.io - flat *-SKILL.md files are
# not discoverable by Claude Code, Codex, Gemini CLI, OpenCode, Hermes, or
# Antigravity). Install the root router the same way.

install_skill() {
  local src="$1" name="$2"
  mkdir -p "$TARGET/$name" || return 1
  cp "$src" "$TARGET/$name/SKILL.md" || return 1
}

if ! install_skill "$REPO_ROOT/SKILL.md" "rustoria"; then
  echo "[FAIL] Could not install root router to $TARGET/rustoria"
  exit 1
fi
echo "[ OK ] Installed root router (rustoria/SKILL.md)"

# Install each skill: skills/<category>/<name>/SKILL.md -> TARGET/<name>/SKILL.md
count=0
for f in $(find skills -name SKILL.md -type f | sort); do
  name=$(tr -d '\r' < "$f" | sed -n 's/^name: *//p' | head -1 | tr -d ' ')
  if [ -z "$name" ]; then
    echo "[WARN] Skipping $f (no name in frontmatter)"
    continue
  fi
  if install_skill "$f" "$name"; then
    count=$((count + 1))
  else
    echo "[FAIL] Could not copy $f to $TARGET/$name"
    exit 1
  fi
done
echo "[ OK ] Installed $count skills"

# Install references if the target supports them.
if [ -d "$REPO_ROOT/references" ]; then
  mkdir -p "$TARGET/references"
  cp "$REPO_ROOT"/references/*.md "$TARGET/references/"
  echo "[ OK ] Installed references"
fi

echo
echo "Installation complete. Skills are in: $TARGET"
echo "Each skill is a directory bundle (<name>/SKILL.md); the root router is rustoria/SKILL.md."
