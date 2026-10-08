#!/usr/bin/env bash
# rustoria repository validator.
# Checks structure, frontmatter, catalog consistency, links, placeholders,
# secrets, personal paths, and workflow integrity. LF/CRLF tolerant.
# Exits non-zero on any failure.
#
# Design rules:
# - A section's "[ OK ]" line is only printed when that section recorded no
#   failures, so green output never masks a [FAIL] above it.
# - Scans exclude */target/* everywhere so results are identical on a fresh
#   clone and after local builds.
# - Portable bash (no associative arrays) so macOS bash 3.2 works too.
set -uo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$REPO_ROOT"

ERRORS=0
WARNINGS=0

fail() { echo "[FAIL] $1"; ERRORS=$((ERRORS + 1)); }
warn() { echo "[WARN] $1"; WARNINGS=$((WARNINGS + 1)); }
pass() { echo "[ OK ] $1"; }

# section_pass <baseline-errors> <message>
# Prints [ OK ] only if no failure was recorded since the baseline.
section_pass() {
  if [ "$ERRORS" -eq "$1" ]; then pass "$2"; fi
}

# Read a file with CR stripped (CRLF tolerant).
strip_cr() { tr -d '\r' < "$1"; }

echo "=== rustoria validation ==="
echo "Repo: $REPO_ROOT"
echo

# ---------------------------------------------------------------- skills ---
echo "--- Skills ---"
BASE=$ERRORS

SKILL_FILES=$(find skills -name SKILL.md -type f | sort)
SKILL_COUNT=$(echo "$SKILL_FILES" | grep -c . )
# Intentionally a hardcoded inventory gate: adding or removing a skill must be
# a reviewed change that updates this file, README, and the root router together.
EXPECTED_COUNT=14

if [ "$SKILL_COUNT" -ne "$EXPECTED_COUNT" ]; then
  fail "Expected $EXPECTED_COUNT skills, found $SKILL_COUNT"
else
  pass "Skill count: $SKILL_COUNT"
fi

SEEN_NAMES=""
for f in $SKILL_FILES; do
  dir=$(basename "$(dirname "$f")")
  content=$(strip_cr "$f")

  # Frontmatter delimiters
  first=$(echo "$content" | head -1)
  if [ "$first" != "---" ]; then
    echo "$f: missing frontmatter start"; fail "$f: missing frontmatter start"
    continue
  fi

  name=$(echo "$content" | sed -n 's/^name: *//p' | head -1 | tr -d ' ')
  desc=$(echo "$content" | sed -n 's/^description: *//p' | head -1)

  if [ -z "$name" ]; then
    fail "$f: missing name in frontmatter"
    continue
  fi

  if [ "$name" != "$dir" ]; then
    fail "$f: name '$name' does not match directory '$dir'"
  fi

  if [ -z "$desc" ]; then
    fail "$f: missing description"
  else
    # Strict YAML: an unquoted ': ' in a plain scalar parses as a nested
    # mapping, which strict YAML parsers (Agent Skills consumers) reject.
    # Quoted values are unquoted here so length/period checks see the value,
    # not the YAML quoting.
    case "$desc" in
      \"*\")
        desc=${desc#\"}
        desc=${desc%\"}
        ;;
      \'*\')
        desc=${desc#\'}
        desc=${desc%\'}
        ;;
      *": "*)
        fail "$f: description contains unquoted ': ' (invalid YAML plain scalar; quote the value)"
        ;;
    esac
    dlen=${#desc}
    if [ "$dlen" -gt 100 ]; then
      fail "$f: description is $dlen chars (max 100)"
    fi
    if ! echo "$desc" | grep -q '\.$'; then
      fail "$f: description does not end with a period"
    fi
  fi

  # Duplicate-name detection without associative arrays (bash 3.2 portable).
  if echo "$SEEN_NAMES" | grep -qx "$name"; then
    fail "Duplicate skill name: $name"
  else
    SEEN_NAMES="${SEEN_NAMES}${name}"$'\n'
  fi
done

# Strict YAML parse of every frontmatter, mirroring what Agent Skills
# consumers do. Runs when python3 + PyYAML are available (CI runners);
# the portable quote check above remains the fallback elsewhere.
if command -v python3 >/dev/null 2>&1 && python3 -c 'import yaml' >/dev/null 2>&1; then
  if ! python3 -c '
import pathlib, sys, yaml
root = pathlib.Path(sys.argv[1])
files = [root / "SKILL.md"] + sorted((root / "skills").rglob("SKILL.md"))
bad = 0
for p in files:
    try:
        text = p.read_text(encoding="utf-8").replace("\r\n", "\n")
        if not text.startswith("---\n"):
            continue
        end = text.find("\n---\n", 4)
        yaml.safe_load(text[4:end])
    except Exception as e:
        print("[FAIL] %s: frontmatter YAML rejected: %s" % (p, e))
        bad += 1
sys.exit(1 if bad else 0)
' "$REPO_ROOT"; then
    fail "Frontmatter YAML rejected by strict parser (see messages above)"
  fi
fi
section_pass "$BASE" "Skill frontmatter and names valid"

# ------------------------------------------------------- phantom references ---
echo
echo "--- Phantom skill references ---"
BASE=$ERRORS

# Only flag backticked (code-formatted) skill names in files that advertise
# current skills. Plain-text historical/deferral mentions are legitimate.
for phantom in rust-cli rust-cloud rust-embedded rust-fintech rust-iot rust-ml rust-web rust-topcoat rust-gpui-kit rust-teacher rust-meta-cognition rust-dynamic-skills; do
  hits=$(grep -l "\`$phantom\`" README.md SKILL.md AGENTS.md tests/trigger-tests.md scripts/test-triggers.sh 2>/dev/null)
  if [ -n "$hits" ]; then
    fail "Phantom skill '$phantom' advertised in: $hits"
  fi
done
section_pass "$BASE" "No phantom skill references in skill catalogs"

# ------------------------------------------------------- README catalog ---
echo
echo "--- README catalog ---"
BASE=$ERRORS

# Extract skill names from catalog table rows only (lines starting with |).
README_SKILLS=$(grep -E '^\|' README.md | grep -oE '`(rust-[a-z-]+)`' | tr -d '`' | sort -u)
for s in $README_SKILLS; do
  if [ ! -f "skills"/*/"$s"/SKILL.md ] && [ ! -f "skills/$s/SKILL.md" ]; then
    # search nested
    found=$(find skills -name SKILL.md -type f -exec grep -l "^name: $s\$" {} \;)
    if [ -z "$found" ]; then
      fail "README catalogs '$s' but no such skill exists"
    fi
  fi
done

# Every skill must be in README
for f in $SKILL_FILES; do
  name=$(strip_cr "$f" | sed -n 's/^name: *//p' | head -1 | tr -d ' ')
  if ! echo "$README_SKILLS" | grep -qx "$name"; then
    fail "Skill '$name' not listed in README catalog"
  fi
done

# AGENTS.md catalog must mention every skill too (agents read AGENTS.md first).
AGENTS_CONTENT=$(strip_cr AGENTS.md 2>/dev/null)
if [ -z "$AGENTS_CONTENT" ]; then
  fail "AGENTS.md missing or unreadable"
else
  for f in $SKILL_FILES; do
    name=$(strip_cr "$f" | sed -n 's/^name: *//p' | head -1 | tr -d ' ')
    if ! echo "$AGENTS_CONTENT" | grep -q "$name"; then
      fail "Skill '$name' not listed in AGENTS.md catalog"
    fi
  done
fi
section_pass "$BASE" "README and AGENTS.md catalogs consistent"

# ------------------------------------------------------- router coverage ---
echo
echo "--- Root router ---"
BASE=$ERRORS

ROOT_SKILL=$(strip_cr SKILL.md)
for f in $SKILL_FILES; do
  name=$(strip_cr "$f" | sed -n 's/^name: *//p' | head -1 | tr -d ' ')
  if [ "$name" != "rustoria" ] && ! echo "$ROOT_SKILL" | grep -q "$name"; then
    fail "Skill '$name' not routed in root SKILL.md"
  fi
done
# The root router must not advertise skills that no longer exist.
for routed in $(echo "$ROOT_SKILL" | grep -oE '`rust-[a-z-]+`' | tr -d '`' | sort -u); do
  if [ "$routed" = "rustoria" ]; then continue; fi
  if ! echo "$SKILL_FILES" | grep -q "/$routed/SKILL.md"; then
    fail "Root router references non-existent skill '$routed'"
  fi
done
section_pass "$BASE" "All skills routed; router has no dead entries"

# ------------------------------------------------------- internal links ---
echo
echo "--- Internal links ---"
BASE=$ERRORS

for f in $(find . -name '*.md' -type f -not -path './.git/*' -not -path '*/target/*'); do
  dir=$(dirname "$f")
  # markdown links to local files (strip #anchors so anchored links are checked)
  links=$(grep -oE '\]\([^)]+\.(md|rs|toml|sh|yml|yaml)(#[^)]+)?\)' "$f" 2>/dev/null \
    | sed 's/](//;s/)$//' | sed 's/#.*$//' | grep -v '^http' | sort -u)
  for l in $links; do
    [ -z "$l" ] && continue
    target="$dir/$l"
    if [ ! -e "$target" ]; then
      fail "$f: broken internal link '$l'"
    fi
  done
done
section_pass "$BASE" "Internal links valid (anchors included)"

# ------------------------------------------------------- placeholders ---
echo
echo "--- Placeholders ---"
BASE=$ERRORS

SCAN_DIRS="skills references docs examples"
SCAN_FILES="README.md AGENTS.md DISCLAIMER.md THIRD-PARTY-NOTICE.md CONTRIBUTING.md CHANGELOG.md SECURITY.md CODE_OF_CONDUCT.md"
for pat in 'your-org' '\[INSERT' 'your-username'; do
  hits=$(grep -rIn "$pat" $SCAN_DIRS $SCAN_FILES 2>/dev/null | grep -v '/target/')
  if [ -n "$hits" ]; then
    fail "Placeholder '$pat' found:"$'\n'"$hits"
  fi
done
# TODO/FIXME/XXX as whole words (avoids matching TODO_STORAGE_PATH etc.)
for pat in TODO FIXME XXX; do
  hits=$(grep -rInw "$pat" $SCAN_DIRS $SCAN_FILES 2>/dev/null | grep -v '/target/')
  if [ -n "$hits" ]; then
    fail "Placeholder '$pat' found:"$'\n'"$hits"
  fi
done
section_pass "$BASE" "No placeholders"

# ------------------------------------------------------- secrets ---
echo
echo "--- Secret files ---"
BASE=$ERRORS

# .env.example is a whitelisted template (matches .gitignore's `!.env.example`)
# and must not be reported as a secret.
for f in $(find . -type f -not -path './.git/*' -not -path '*/target/*' \
  \( -name '.env' -o -name '.env.*' -o -name '*.pem' -o -name '*.key' -o -name 'id_rsa*' -o -name '*.p12' -o -name '*.pfx' \) \
  ! -name '.env.example'); do
  fail "Secret file present: $f"
done
section_pass "$BASE" "No secret files"

# ------------------------------------------------------- personal paths ---
echo
echo "--- Personal paths ---"
BASE=$ERRORS

hits=$(grep -rIn -e 'C:\\Users\\' -e '/Users/' -e '/home/' -e '\.cargo\\registry' -e '\.rustup' $SCAN_DIRS $SCAN_FILES 2>/dev/null | grep -v '/target/')
if [ -n "$hits" ]; then
  fail "Personal paths found:"$'\n'"$hits"
fi
section_pass "$BASE" "No personal paths"

# ------------------------------------------------------- examples ---
echo
echo "--- Examples ---"
BASE=$ERRORS

for d in examples/*/; do
  if [ -d "$d" ]; then
    if [ ! -f "$d/Cargo.toml" ]; then
      fail "Example '$d' missing Cargo.toml"
    fi
  fi
done
section_pass "$BASE" "Examples have Cargo.toml"

# ------------------------------------------------------- references ---
echo
echo "--- References ---"
BASE=$ERRORS

REF_COUNT=$(find references -name '*.md' -type f | grep -c .)
if [ "$REF_COUNT" -lt 12 ]; then
  warn "Expected at least 12 reference files, found $REF_COUNT"
else
  pass "Reference count: $REF_COUNT"
fi

# ------------------------------------------------------- workflows ---
echo
echo "--- Workflows ---"
BASE=$ERRORS

for w in .github/workflows/*.yml; do
  if [ -f "$w" ]; then
    if ! grep -q 'uses:' "$w" && ! grep -q 'run:' "$w"; then
      fail "Workflow '$w' has no steps"
    fi
    # Every workflow must declare least-privilege token permissions explicitly.
    if ! grep -q '^permissions:' "$w" && ! grep -q '^[[:space:]]\+permissions:' "$w"; then
      fail "Workflow '$w' has no explicit permissions: block"
    fi
    # Third-party actions must be pinned to a commit SHA, not a mutable tag.
    unpinned=$(grep -E '^[[:space:]]*- uses:' "$w" | grep -vE '@[0-9a-f]{40}' || true)
    if [ -n "$unpinned" ]; then
      fail "Workflow '$w' has actions not pinned to a SHA:"$'\n'"$unpinned"
    fi
    # Untrusted interpolation inside run: blocks is a script-injection risk.
    # Scan scalar run: lines and multi-line run: | blocks; only workflow-authored
    # ${{ matrix.* }} values are allowed there (pass trusted values via env:).
    bad_interp=$(awk '
      {
        if (inrun && $0 ~ /[^ ]/) {
          first = match($0, /[^ ]/)
          if (first <= runindent) inrun = 0
        }
        if (!inrun && $0 ~ /^[[:space:]]*#/) next
        if ($0 ~ /run:[[:space:]]*\|/) {
          inrun = 1; runindent = match($0, /[^ ]/); next
        }
        if ($0 ~ /run:.*\$\{\{/ || (inrun && $0 ~ /\$\{\{/)) {
          line = $0
          gsub(/\$\{\{[[:space:]]*matrix\.[^}]*\}\}/, "", line)
          if (line ~ /\$\{\{/) print FNR ": " $0
        }
      }' "$w")
    if [ -n "$bad_interp" ]; then
      fail "Workflow '$w' interpolates untrusted context into a run: block:"$'\n'"$bad_interp"
    fi
  fi
done
section_pass "$BASE" "Workflows valid (permissions, pinning, interpolation)"

# ------------------------------------------------------- summary ---
echo
echo "=== Summary ==="
echo "Errors: $ERRORS  Warnings: $WARNINGS"
if [ "$ERRORS" -gt 0 ]; then
  exit 1
fi
exit 0
