# Antigravity Integration

Antigravity discovers skills from **`<workspace-root>/.agents/skills/<skill-folder>/`**
(workspace scope) and **`~/.gemini/config/skills/<skill-folder>/`** (global scope; the
legacy `~/.gemini/antigravity/skills/` is also supported). Source: the
[Antigravity agent skills documentation](https://antigravity.google/docs/skills/).

> There is no documented `~/.antigravity/skills` discovery path.

## Installation

### Option 1: Script (recommended, global scope)

```bash
# From the repository root
bash scripts/setup.sh ~/.gemini/config/skills
```

### Option 2: Manual (workspace scope, shared with your team)

```bash
mkdir -p .agents/skills/rustoria
cp SKILL.md .agents/skills/rustoria/SKILL.md
for f in $(find skills -name SKILL.md); do
  name=$(grep '^name:' "$f" | head -1 | sed 's/name: *//')
  mkdir -p .agents/skills/$name
  cp "$f" .agents/skills/$name/SKILL.md
done
```

Each skill must be a directory containing `SKILL.md` - flat `*-SKILL.md` files are not
discovered. Skills follow the [Agent Skills standard](https://agentskills.io).

## Usage

Reference skills in your Antigravity prompts:

```text
Use the rustoria skill for [your Rust task]
```

Each skill also becomes a slash command (`/rust-fundamentals`) in the CLI/IDE.

## Notes

- Antigravity defaults to `.agents/skills` and keeps backward compatibility with the
  older `.agent/skills`
- The root router is the `rustoria` skill; it routes to specific skills
