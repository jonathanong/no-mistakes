# `github-actions-no-insta-snapshot-updates`

## Why and when

CI should fail when a committed snapshot is missing or differs from the
test output. A workflow that lets Insta accept snapshots can turn a change
that needs review into a passing check.

## What it catches

The rule checks workflow, job, and step `env` mappings and step `run:`
scripts in files directly under `.github/workflows/`.

- Literal values of `always`, `1`, `force`, and `unseen` are reported as
  findings pointing to the declaration that enables snapshot acceptance.
  `unseen` accepts missing snapshots while leaving changed snapshots pending
  review.
- In `run:` scripts, it flags `cargo insta accept` (alias `approve`) and
  `cargo insta test` with `--accept`, `--accept-unseen`, or
  `--force-update-snapshots`. Direct `cargo-insta` invocations are covered
  too. Arguments after `--` belong to the test runner. Findings point to the
  step's `run:` key.
- Opaque `run:` scripts use the same unanalyzable policy. Examples include
  GitHub expressions interpolated into the script, dynamic command or Insta
  arguments, incomplete shell syntax, and unsupported shells.
- Other inputs the rule cannot decide statically — unreadable or non-UTF-8 workflow
  files, unparsable workflow YAML (reported at the parser's line), non-mapping
  workflow documents, `jobs`, jobs, steps, or `env:` values (such as
  `jobs: ${{ fromJSON(vars.JOBS) }}`, `env: ${{ fromJSON(vars.ENV) }}`, or
  `env: []`), non-sequence `steps:` values, YAML merge keys (`<<:`), non-scalar
  `INSTA_UPDATE` values, or GitHub expressions such as
  `${{ vars.SNAPSHOT_MODE }}` — are routed through the rule's shared
  `unanalyzable` policy (`report` by default, or `advise` / `ignore`).

## Options and defaults

This rule is off by default. Add it to `rules` to enable it. It honors the
rule's standard `include`, `exclude`, and `message` fields and the shared
`options.unanalyzable` policy (`report` by default, or `advise` / `ignore`).

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
      - run: cargo insta test
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
      - run: cargo insta test --accept
      - run: cargo insta accept
```

## Fix

Set `INSTA_UPDATE` to `"no"` in CI and run `cargo insta test` without
acceptance flags. Review and commit deliberate snapshot changes separately.

## Suppression

For a dedicated snapshot-maintenance workflow, use the standard YAML
directive `# no-mistakes-disable-file github-actions-no-insta-snapshot-updates`.
For a single declaration use `no-mistakes-disable-line` or
`no-mistakes-disable-next-line` with the same rule name.

## Limits

The shell analysis handles direct commands in Bash and POSIX shell steps.
Shell scripts invoked from separate files, inline shell `INSTA_UPDATE=...`
assignments, local composite actions, and standalone Insta configuration are
outside the rule's scope. The environment check covers the declaration at
each scope, including one overridden by a job or step. The rule is specific
to Insta. It leaves Vitest and Jest `-u` alone.

See [Insta's snapshot update modes](https://insta.rs/docs/advanced/#controlling-snapshot-updating)
for the runner's behavior.

## Related rules

Use [GitHub Actions run command policy](github-actions-run-command-policy.md)
to forbid snapshot-updating flags for other tools in `run:` steps. Use
[GitHub Actions environment policy](github-actions-env-policy.md) to require
one exact value wherever an environment variable is defined.
