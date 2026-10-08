# OpenCode Integration

OpenCode discovers skills as **directories** from, among other locations:

- Project: `.opencode/skills/<name>/SKILL.md`
- Global: `~/.config/opencode/skills/<name>/SKILL.md`
- Claude/agent-compatible aliases: `.claude/skills/`, `~/.claude/skills/`,
  `.agents/skills/`, `~/.agents/skills/`

Source: the [OpenCode skills documentation](https://opencode.ai/docs/skills/).

> There is no documented `~/.opencode/skills` discovery path.

## Installation

### Option 1: Script (recommended, global scope)

```bash
# From the repository root
bash scripts/setup.sh ~/.config/opencode/skills
```

### Option 2: Manual (project scope, shared via version control)

```bash
mkdir -p .opencode/skills/rustoria
cp SKILL.md .opencode/skills/rustoria/SKILL.md
for f in $(find skills -name SKILL.md); do
  name=$(grep '^name:' "$f" | head -1 | sed 's/name: *//')
  mkdir -p .opencode/skills/$name
  cp "$f" .opencode/skills/$name/SKILL.md
done
```

OpenCode requires the skill's frontmatter `name` to match its directory name - the
rustoria names (`rust-fundamentals`, `rust-std`, ...) already do.

## Usage

Reference skills in your OpenCode prompts:

```text
Use the rustoria skill for [your Rust task]
```

Skills are loaded on demand through the native skill tool.

## Notes

- Each skill is a directory bundle (`<name>/SKILL.md`), per the
  [Agent Skills standard](https://agentskills.io)
- The root router is the `rustoria` skill; it routes to specific skills
- For project-local paths, OpenCode walks up from the current working directory until it
  reaches the git worktree, loading matching skills along the way
