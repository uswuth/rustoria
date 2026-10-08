# OpenAI Codex Integration

Codex discovers skills from **`.agents/skills`** directories (repository and user scope)
plus `/etc/codex/skills` for admin scope. The user-level location is `~/.agents/skills`.
Source: the [Codex skills documentation](https://developers.openai.com/codex/skills).

> There is no documented `~/.codex/skills` discovery path - `~/.codex/` holds
> `config.toml`, not skills.

## Installation

### Option 1: Script (recommended)

```bash
# From the repository root
bash scripts/setup.sh ~/.agents/skills
```

### Option 2: Manual

```bash
mkdir -p ~/.agents/skills/rustoria
cp SKILL.md ~/.agents/skills/rustoria/SKILL.md
for f in $(find skills -name SKILL.md); do
  name=$(grep '^name:' "$f" | head -1 | sed 's/name: *//')
  mkdir -p ~/.agents/skills/$name
  cp "$f" ~/.agents/skills/$name/SKILL.md
done
```

### Option 3: Checked into a repository (team scope)

Copy the same `<name>/SKILL.md` bundles into `.agents/skills/` at the repository root so
every team member's Codex session picks them up.

## Usage

Reference skills in your Codex prompts:

```text
Use the rustoria skill for [your Rust task]
```

## Notes

- Codex lists each skill's name, description, and file path; skills load on demand
- Each skill is a directory bundle (`<name>/SKILL.md`), per the
  [Agent Skills standard](https://agentskills.io)
- The root router is the `rustoria` skill; it routes to specific skills
- The `~/.agents/skills` path is also read by Gemini CLI, OpenCode, and other
  standard-compatible tools, so one install can serve several agents
