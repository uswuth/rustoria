# Claude Code Integration

Claude Code discovers skills as **directories**: `~/.claude/skills/<skill-name>/SKILL.md`
(project scope: `.claude/skills/<skill-name>/SKILL.md`). Source: the
[Claude Code skills reference](https://code.claude.com/docs/en/skills).

## Installation

### Option 1: Script (recommended)

```bash
# From the repository root
bash scripts/setup.sh ~/.claude/skills
```

### Option 2: Manual

```bash
mkdir -p ~/.claude/skills/rustoria
cp SKILL.md ~/.claude/skills/rustoria/SKILL.md
for f in $(find skills -name SKILL.md); do
  name=$(grep '^name:' "$f" | head -1 | sed 's/name: *//')
  mkdir -p ~/.claude/skills/$name
  cp "$f" ~/.claude/skills/$name/SKILL.md
done
```

Flat `*-SKILL.md` files are **not** discovered by Claude Code - each skill must be
its own directory containing `SKILL.md`.

## Usage

Load the root router for any Rust task:

```text
Load the rustoria skill and help me with [your Rust task]
```

The router will direct you to the appropriate skill. Skills are also invoked directly
by name (for example `/rust-fundamentals`).

## What is included

- 14 skills (6 native core + 1 security + 7 library extensions)
- 12 reference documents
- 6 compile-tested examples

## Notes

- The directory name (or the frontmatter `name`) becomes the command you type
- Skill names match the directory names (`rust-fundamentals`, `rust-std`, ...)
- References live in the repository `references/` directory - clone the repository
  alongside your projects for deep dives
