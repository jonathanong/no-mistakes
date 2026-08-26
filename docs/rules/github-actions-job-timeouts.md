# `github-actions-job-timeouts`

## Why and when

Use this rule when every executable job needs a bounded failure time instead of
GitHub's long default ceiling.

## What it requires

It requires a literal job `timeout-minutes`, optionally caps it, and can reject
a literal step timeout that exceeds its containing job timeout.

## Options

`include` defaults to workflow `.yml` and `.yaml` files. `maxMinutes` is an
optional cap, `rejectStepExceedingJob` defaults to `false`, and `allow` entries
contain a `path#jobId` plus optional higher `maxMinutes`.

## Valid example

The fixed `test` job below is valid because its literal timeout is within the
configured cap.

## Related rules

[`github-actions-test-timeout-literals`](github-actions-test-timeout-literals.md)
keeps tests from duplicating this policy, while
[`github-actions-action-timeout-pair`](github-actions-action-timeout-pair.md)
covers nested action limits.

## Suppression

Use a line directive for one job or a file directive for an exceptional
workflow. Prefer a reasoned `allow` entry for a known job that legitimately
needs a higher cap.

Requires each GitHub Actions job to set a literal `timeout-minutes` so the
job cannot sit on the default 6-hour ceiling. Optional `maxMinutes` caps that
literal. Jobs that call a reusable workflow with `uses:` are skipped because
GitHub rejects caller-side `timeout-minutes`.

```yaml
rules:
  - rule: github-actions-job-timeouts
    scope: repository
    options:
      include: [".github/workflows/**/*.yml"]
      maxMinutes: 10
      rejectStepExceedingJob: true
      allow:
        - job: ".github/workflows/ci.yml#coverage"
          maxMinutes: 20
```

`include` defaults to `.github/workflows/**/*.yml` and `*.yaml`.
`timeout-minutes` must be a number or numeric string. Expression forms such as
`fromJSON(vars…)` are not resolved; put those jobs in `allow`.
`rejectStepExceedingJob` also flags a step whose literal timeout exceeds its
job timeout. Invalid YAML is a diagnostic. Unused `allow` entries for jobs in
the scanned file are findings.

Counterexample: a job has no timeout, or sets `timeout-minutes: 30` when
`maxMinutes` is 10.

```yaml
jobs:
  test:
    runs-on: ubuntu-slim
    steps:
      - run: echo hi
```

Fix: set a literal timeout at or below the cap, or add `path#jobId` to
`allow` with a higher `maxMinutes`.

```yaml
jobs:
  test:
    runs-on: ubuntu-slim
    timeout-minutes: 10
    steps:
      - run: echo hi
```

Use `no-mistakes-disable-next-line github-actions-job-timeouts` or
`no-mistakes-disable-file` for a one-off workflow exception.
