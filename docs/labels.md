# Label Taxonomy

Labels are used to categorize issues and PRs. Do not create labels merely for decoration.

## Types

| Label | Description |
| ------- | ------------- |
| `type:bug` | Incorrect information, broken links, non-compiling examples |
| `type:feature` | New capability or skill |
| `type:skill` | Skill content change |
| `type:documentation` | Documentation change |
| `type:maintenance` | Maintenance task |
| `type:security` | Security issue |

## Areas

| Label | Description |
| ------- | ------------- |
| `area:rust-core` | Rust language/compiler |
| `area:std` | Standard library |
| `area:unsafe` | Unsafe Rust |
| `area:cargo` | Cargo/toolchain |
| `area:library` | Third-party crates |
| `area:agent` | Agent integration |
| `area:automation` | Automation/CI |
| `area:ci` | CI/CD |
| `area:docs` | Documentation |

## Priority

| Label | Description |
| ------- | ------------- |
| `priority:critical` | Blocking issue |
| `priority:high` | Important |
| `priority:medium` | Normal |
| `priority:low` | Minor |

## Status

| Label | Description |
| ------- | ------------- |
| `status:triage` | Needs review |
| `status:planned` | Accepted, not started |
| `status:blocked` | Blocked by dependency |
| `status:in-progress` | Being worked on |

## Special

| Label | Description |
| ------- | ------------- |
| `good-first-issue` | Suitable for new contributors |
| `help-wanted` | Needs community help |
| `breaking-change` | Breaks compatibility |
| `deferred` | Intentionally postponed |

## Usage

- Every issue should have at least one `type:` and one `area:` label.
- Priority labels are optional but recommended for bugs.
- Status labels are applied during triage.
- `deferred` is used for intentionally postponed work (e.g. gpui-kit, topcoat).

<!-- PR automation test artifact; this branch is never merged -->