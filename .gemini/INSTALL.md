# Gemini CLI Integration

Gemini CLI discovers skills from **`~/.gemini/skills/`** (user scope) or the
`~/.agents/skills/` alias; workspace skills live in `.gemini/skills/` or
`.agents/skills/`. Source: the
[Gemini CLI Agent Skills documentation](https://geminicli.com/docs/cli/skills/).

## Installation

### Option 1: Script (recommended)

```bash
# From the repository root
bash scripts/setup.sh ~/.gemini/skills
```

### Option 2: Manual

```bash
mkdir -p ~/.gemini/skills/rustoria
cp SKILL.md ~/.gemini/skills/rustoria/SKILL.md
for f in $(find skills -name SKILL.md); do
  name=$(grep '^name:' "$f" | head -1 | sed 's/name: *//')
  mkdir -p ~/.gemini/skills/$name
  cp "$f" ~/.gemini/skills/$name/SKILL.md
done
```

Each skill must be its own directory containing `SKILL.md` - flat `*-SKILL.md` files are
not discovered. Gemini CLI asks for consent before injecting a skill's contents.

## Usage

Reference skills in your Gemini prompts:

```text
Use the rustoria skill for [your Rust task]
```

Or list what is loaded with `/skills list` inside a session.

## Notes

- Skills follow the [Agent Skills standard](https://agentskills.io) - a folder with a
  `SKILL.md` inside; the `.agents/skills/` alias takes precedence over `.gemini/skills/`
  within the same tier
- The root router is the `rustoria` skill; it routes to specific skills
- Gemini CLI activates skills by matching task text against each skill's `description`
