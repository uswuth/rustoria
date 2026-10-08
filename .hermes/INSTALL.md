# Hermes Integration

Hermes discovers skills from **`~/.hermes/skills/`** (the primary directory and source
of truth), and also scans `.agents/skills/` alongside it. Source: the
[Hermes skills documentation](https://hermes-agent.nousresearch.com/docs/user-guide/features/skills).

## Installation

### Option 1: Script (recommended)

```bash
# From the repository root
bash scripts/setup.sh ~/.hermes/skills
```

### Option 2: Manual

```bash
mkdir -p ~/.hermes/skills/rustoria
cp SKILL.md ~/.hermes/skills/rustoria/SKILL.md
for f in $(find skills -name SKILL.md); do
  name=$(grep '^name:' "$f" | head -1 | sed 's/name: *//')
  mkdir -p ~/.hermes/skills/$name
  cp "$f" ~/.hermes/skills/$name/SKILL.md
done
```

Each skill must be a directory containing `SKILL.md` (Hermes follows the
[Agent Skills standard](https://agentskills.io)); flat `*-SKILL.md` files are not
discovered.

## Usage

Load the root router for any Rust task. The router directs to the appropriate skill.

## Notes

- Hermes can modify skills it owns (including `/learn`-authored ones) - keep your own
  copy in version control; the `rustoria` repository is the source of truth
- References are in the repository `references/` directory - clone the repository
  alongside your projects for deep dives
- The root router (the `rustoria` skill) routes to specific skills
