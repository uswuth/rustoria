#!/usr/bin/env bash
# Canonical trigger-test harness for rustoria.
#
# Single source of truth: tests/trigger-tests.md tables are parsed directly,
# so the documented cases and the executed cases cannot drift apart.
#
#   - positive:    expected skill exists and its description contains at least
#                  one of the case's routing keywords
#   - negative:    no skill description contains any of the case's keywords
#   - composition: every skill:keyword pair matches
#   - router:      root SKILL.md routes every skill and has no dead entries
#
# LF/CRLF tolerant. Exits non-zero on any failure.
set -uo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$REPO_ROOT"

ERRORS=0
PASSED=0
ROUTER_CHECKS=0

strip_cr() { tr -d '\r' < "$1"; }
trim() { sed 's/^[[:space:]]*//;s/[[:space:]]*$//' <<< "$1"; }

SPEC=tests/trigger-tests.md
if [ ! -f "$SPEC" ]; then
  echo "[FAIL] missing $SPEC (single source of truth for trigger cases)"
  exit 1
fi

# Load all skill descriptions from the real SKILL.md files.
SKILL_NAMES=()
SKILL_DESCS=()
while IFS= read -r f; do
  content=$(strip_cr "$f")
  name=$(echo "$content" | sed -n 's/^name: *//p' | head -1 | tr -d ' ')
  desc=$(echo "$content" | sed -n 's/^description: *//p' | head -1)
  SKILL_NAMES+=("$name")
  SKILL_DESCS+=("$desc")
done < <(find skills -name SKILL.md -type f | sort)

skill_count=${#SKILL_NAMES[@]}
echo "=== rustoria trigger tests ==="
echo "Loaded $skill_count skills from skills/**/SKILL.md"
echo "Cases parsed from $SPEC"
echo

skill_exists() {
  local want="$1"
  for n in "${SKILL_NAMES[@]}"; do
    if [ "$n" = "$want" ]; then return 0; fi
  done
  return 1
}

skill_desc() {
  local want="$1"
  for i in "${!SKILL_NAMES[@]}"; do
    if [ "${SKILL_NAMES[$i]}" = "$want" ]; then
      echo "${SKILL_DESCS[$i]}"
      return 0
    fi
  done
  return 1
}

# Case-insensitive substring match.
desc_has() {
  local desc="$1" kw="$2"
  echo "$desc" | tr '[:upper:]' '[:lower:]' | grep -qi -- "$kw"
}

# Extract table rows (excluding header/separator) from a "## <section>" section.
table_rows() {
  local section="$1"
  awk -v hdr="$section" '
    index($0, "## ") == 1 { f = ($0 ~ "^## " hdr); next }
    f && /^\|/ { sub(/\r$/, ""); print }
  ' "$SPEC"
}

# ---------------------------------------------------------------- cases ---
POSITIVE_ROWS=$(table_rows "Positive cases" | grep -c '^| TC-')
NEGATIVE_ROWS=$(table_rows "Negative cases" | grep -c '^| TC-')
COMPOSITION_ROWS=$(table_rows "Composition cases" | grep -c '^| TC-')

if [ "$POSITIVE_ROWS" -eq 0 ] || [ "$NEGATIVE_ROWS" -eq 0 ] || [ "$COMPOSITION_ROWS" -eq 0 ]; then
  echo "[FAIL] could not parse case tables from $SPEC (positive=$POSITIVE_ROWS negative=$NEGATIVE_ROWS composition=$COMPOSITION_ROWS)"
  exit 1
fi

# Positive: expected skill exists and description contains at least one keyword.
while IFS='|' read -r _ id prompt skill kws _; do
  id=$(trim "$id")
  case "$id" in TC-*) ;; *) continue ;; esac
  prompt=$(trim "$prompt")
  skill=$(trim "$skill" | tr -d '`')
  kws=$(trim "$kws")

  if ! skill_exists "$skill"; then
    echo "[FAIL] $id: skill '$skill' does not exist"
    ERRORS=$((ERRORS + 1))
    continue
  fi
  d=$(skill_desc "$skill")

  matched=0
  IFS=',' read -ra kw_arr <<< "$kws"
  for kw in "${kw_arr[@]}"; do
    kw=$(trim "$kw")
    [ -z "$kw" ] && continue
    if desc_has "$d" "$kw"; then
      matched=1
      break
    fi
  done

  if [ "$matched" -eq 1 ]; then
    echo "[ OK ] $id: '$skill' matches '$kw' ($prompt)"
    PASSED=$((PASSED + 1))
  else
    echo "[FAIL] $id: '$skill' description matches none of [$kws]"
    echo "       desc: $d"
    ERRORS=$((ERRORS + 1))
  fi
done < <(table_rows "Positive cases")

# Negative: no skill description contains any of the keywords.
while IFS='|' read -r _ id prompt kws why _; do
  id=$(trim "$id")
  case "$id" in TC-*) ;; *) continue ;; esac
  prompt=$(trim "$prompt")
  kws=$(trim "$kws")

  failed=0
  IFS=',' read -ra kw_arr <<< "$kws"
  for kw in "${kw_arr[@]}"; do
    kw=$(trim "$kw")
    [ -z "$kw" ] && continue
    matched=()
    for i in "${!SKILL_NAMES[@]}"; do
      if desc_has "${SKILL_DESCS[$i]}" "$kw"; then
        matched+=("${SKILL_NAMES[$i]}")
      fi
    done
    if [ ${#matched[@]} -gt 0 ]; then
      echo "[FAIL] $id: keyword '$kw' matches skills ${matched[*]} ($prompt)"
      failed=1
    fi
  done

  if [ "$failed" -eq 0 ]; then
    echo "[ OK ] $id: no skill matches [$kws] ($prompt)"
    PASSED=$((PASSED + 1))
  else
    ERRORS=$((ERRORS + 1))
  fi
done < <(table_rows "Negative cases")

# Composition: every skill:keyword pair must match.
while IFS='|' read -r _ id prompt pairs _; do
  id=$(trim "$id")
  case "$id" in TC-*) ;; *) continue ;; esac
  prompt=$(trim "$prompt")
  pairs=$(trim "$pairs")

  ok=1
  IFS=',' read -ra pair_arr <<< "$pairs"
  for pair in "${pair_arr[@]}"; do
    pair=$(trim "$pair")
    skill=$(trim "${pair%%:*}" | tr -d '`')
    kw=$(trim "${pair#*:}")
    if ! skill_exists "$skill"; then
      echo "[FAIL] $id: skill '$skill' does not exist"
      ok=0
      continue
    fi
    d=$(skill_desc "$skill")
    if ! desc_has "$d" "$kw"; then
      echo "[FAIL] $id: '$skill' does not match '$kw'"
      ok=0
    fi
  done

  if [ "$ok" -eq 1 ]; then
    echo "[ OK ] $id: composition ($prompt)"
    PASSED=$((PASSED + 1))
  else
    ERRORS=$((ERRORS + 1))
  fi
done < <(table_rows "Composition cases")

# ---------------------------------------------------------------- router ---
# The root router is the real entry point: verify both directions.
echo
echo "--- Root router ---"

if [ ! -f SKILL.md ]; then
  echo "[FAIL] root SKILL.md missing"
  ERRORS=$((ERRORS + 1))
else
  ROOT_NAME=$(strip_cr SKILL.md | sed -n 's/^name: *//p' | head -1 | tr -d ' ')
  if [ "$ROOT_NAME" = "rustoria" ]; then
    echo "[ OK ] router frontmatter name is 'rustoria'"
    ROUTER_CHECKS=$((ROUTER_CHECKS + 1))
  else
    echo "[FAIL] router frontmatter name is '$ROOT_NAME', expected 'rustoria'"
    ERRORS=$((ERRORS + 1))
  fi

  ROOT_CONTENT=$(strip_cr SKILL.md)
  for n in "${SKILL_NAMES[@]}"; do
    if echo "$ROOT_CONTENT" | grep -q "$n"; then
      ROUTER_CHECKS=$((ROUTER_CHECKS + 1))
    else
      echo "[FAIL] router does not reference skill '$n'"
      ERRORS=$((ERRORS + 1))
    fi
  done

  # Every skill-like token in the router must resolve to a real skill.
  for routed in $(echo "$ROOT_CONTENT" | grep -oE '`rust-[a-z-]+`' | tr -d '`' | sort -u); do
    if ! skill_exists "$routed"; then
      echo "[FAIL] router references non-existent skill '$routed'"
      ERRORS=$((ERRORS + 1))
    else
      ROUTER_CHECKS=$((ROUTER_CHECKS + 1))
    fi
  done
fi

# ---------------------------------------------------------- summary ---
echo
echo "=== Summary ==="
echo "Case tests passed: $PASSED / $((POSITIVE_ROWS + NEGATIVE_ROWS + COMPOSITION_ROWS))"
echo "Router assertions passed: $ROUTER_CHECKS"
echo "Failed: $ERRORS"
if [ "$ERRORS" -gt 0 ]; then
  exit 1
fi
exit 0
