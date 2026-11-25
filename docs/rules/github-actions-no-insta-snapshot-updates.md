# `github-actions-no-insta-snapshot-updates`

## Why and when

CI should fail when a committed snapshot is missing or differs from the
test output. A workflow that lets Insta accept snapshots can turn a change
that needs review into a passing check.

## What it catches

The rule reports literal `INSTA_UPDATE` settings of `always`, `1`, `force`,
and `unseen` in workflow, job, and step `env` mappings. The finding points
to the declaration that enables snapshot acceptance. `unseen` accepts
missing snapshots while leaving changed snapshots pending review.

## Options and defaults

This rule is off by default. Add it to `rules` to enable it. It has no
rule-specific options and honors the rule's standard `include` and `exclude`
scopes.

```yaml
rules:
  - rule: github-actions-no-insta-snapshot-updates
    scope: repository
```

## Valid example

```yaml
name: Tests
on: [pull_request]
jobs:
  unit:
    runs-on: ubuntu-latest
    steps:
      - run: cargo test
        env:
          INSTA_UPDATE: "no"
```

An omitted setting and the literal values `auto` and `new` also pass.
Insta's default `auto` checks snapshots in CI. The `new` mode can create
pending `.snap.new` files while leaving the snapshot mismatch visible.

## Counterexample

```yaml
name: Tests
on: [pull_request]
jobs:
  unit:
    runs-on: ubuntu-latest
    steps:
      - run: cargo test
        env:
          INSTA_UPDATE: always
```

## Fix

Set `INSTA_UPDATE` to `"no"` in CI. Review and commit deliberate snapshot
changes separately.

## Suppression

For a dedicated snapshot-maintenance workflow, use the standard YAML
directive `# no-mistakes-disable-file github-actions-no-insta-snapshot-updates`.
For a single declaration use `no-mistakes-disable-line` or
`no-mistakes-disable-next-line` with the same rule name.

## Limits

The rule reads literal declarations in YAML files directly under
`.github/workflows/`. It leaves dynamic GitHub expressions, local composite
actions, shell assignments, and standalone Insta configuration to other
checks or review. It checks the declaration at each scope, including one
overridden by a job or step. Malformed workflows are handled by workflow
validation.

Use [GitHub Actions environment policy](github-actions-env-policy.md) to
require one exact value wherever the variable is defined. See
[Insta's snapshot update modes](https://insta.rs/docs/advanced/#controlling-snapshot-updating)
for the runner's behavior.
