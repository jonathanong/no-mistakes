# `github-actions-local-action-checkout`

## Why and when

Local actions run from the runner workspace. Enable this rule to catch a job
that uses a repository action before checking out the repository containing it.
Checking out in another job does not prepare this job's workspace.

## What it catches

The rule checks ordered job steps in selected direct `.github/workflows/*.yml`
and `.github/workflows/*.yaml` files. Each local `uses: ./path` needs a preceding
unconditional root checkout of the current repository that covers that path.
Readiness is tracked separately for each action path and resets for each job.
A later checkout cannot satisfy an earlier use.

A checkout uses `actions/checkout@...`. Its `with.path` must be absent, `.`, or
`./`; its `with.repository` must be absent or exactly `${{ github.repository }}`.
A missing `sparse-checkout` covers every local action. Otherwise, a literal
directory prefix must cover the action path: `.github/actions` covers
`.github/actions/setup`, but `.github/other` does not. Expressions and patterns
do not establish directory coverage. A sparse list containing any negated entry
(`!path`) cannot establish coverage, even if another entry is a literal ancestor;
use a positive directory list so coverage is statically provable.

An unconditional root checkout invalidates readiness for every local action
path if `with.repository` names another repository or does not identify the
current repository statically. This includes a different expression or an
explicit null. [Readiness tracking](https://github.com/jonathanong/no-mistakes/blob/cc505a7c1e9c3cb6ad6d69cce53633b67d0c999e/crates/no-mistakes/src/codebase/rules/github_actions_local_action_checkout/scan.rs#L53-L63) and [repository selection](https://github.com/jonathanong/no-mistakes/blob/cc505a7c1e9c3cb6ad6d69cce53633b67d0c999e/crates/no-mistakes/src/codebase/rules/github_actions_local_action_checkout/scan.rs#L84-L89).
Conditional checkouts, checkouts into another directory, and self checkouts
that do not cover an action path leave that path's previous state unchanged.
A conditional self checkout therefore cannot prepare a previously unprepared
local action.

## Configuration

Use the shared `options.unanalyzable` setting to control uncertainty. The
shared `scope`, `projects`, `include`,
and `exclude` fields select applications. By default, direct workflow YAML
files under `.github/workflows/` are selected. The shared `message` field
customizes finding text. This rule does not automatically fix workflows.

```yaml
rules:
  - rule: github-actions-local-action-checkout
    scope: repository
```

## Valid example

```yaml
jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: ./.github/actions/setup
```

## Counterexample

```yaml
jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - uses: ./.github/actions/setup
      - uses: actions/checkout@v4
```

## Fix

Move an unconditional checkout of the current repository before the local
`uses:` step. Keep the checkout at the workspace root. If using sparse
checkout, include a literal directory prefix covering the local action.
After a foreign root checkout, restore the current repository before using
its local actions again.

## Analysis

### Effort

Runs at `max` only. Configuring `effort: low` is accepted: `check` emits an analysis warning (requested effort low; effective effort max because this rule has no low implementation) and runs the rule at `max`.

### Unanalyzable inputs

A missing or non-mapping `jobs`, invalid YAML, and unreadable or non-UTF-8
workflow files follow `options.unanalyzable`. The option inherits the
repository policy and defaults to `report`. [Workflow shape check](https://github.com/jonathanong/no-mistakes/blob/cc505a7c1e9c3cb6ad6d69cce53633b67d0c999e/crates/no-mistakes/src/codebase/rules/github_actions_local_action_checkout/scan.rs#L14-L19), [read and parse diagnostics](https://github.com/jonathanong/no-mistakes/blob/cc505a7c1e9c3cb6ad6d69cce53633b67d0c999e/crates/no-mistakes/src/codebase/rules/github_actions_local_action_checkout/diagnostics.rs#L47-L79), and [policy precedence](https://github.com/jonathanong/no-mistakes/blob/cc505a7c1e9c3cb6ad6d69cce53633b67d0c999e/crates/no-mistakes/src/config/v2/analysis_policy.rs#L86-L96).

## Suppression

Findings point to the local action's `uses:` line. Use
`no-mistakes-disable-line` on that line, `no-mistakes-disable-next-line`
immediately above it, or `no-mistakes-disable-file` for the workflow.

```yaml
jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      # no-mistakes-disable-next-line github-actions-local-action-checkout
      - uses: ./.github/actions/setup
```

## Related rules

[`github-actions-checkout-credentials`](github-actions-checkout-credentials.md)
checks credential persistence. [`github-actions-pinned-hash`](github-actions-pinned-hash.md)
checks immutable action references; enable it separately to require checkout pins.
