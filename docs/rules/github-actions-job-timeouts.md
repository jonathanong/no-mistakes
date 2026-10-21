# `github-actions-job-timeouts`

## Why and when

Use this rule to keep GitHub Actions jobs from running up to GitHub's default
six-hour limit and to ensure the job deadline leaves room for its steps.

## What it requires

Each executable job must set a supported literal `timeout-minutes`. Jobs that
call reusable workflows with `uses:` are skipped because GitHub rejects a
caller-side job timeout. Expressions such as `fromJSON(vars…)` do not satisfy
the literal requirement. An `allow` entry can raise a cap for a job with a
supported literal timeout, but cannot exempt a job from requiring one.

With `rejectStepExceedingJob`, the rule also reports a step whose literal
timeout exceeds its job timeout. With `rejectJobAtOrAboveStepSum`, it reports a
job timeout greater than or equal to the sum of step timeouts, when there are
at least two steps and every step has a supported literal upper bound accepted
by the rule's existing `timeout-minutes` parser. Missing, invalid, or malformed
step timeouts make the sum unprovable, so the sum check skips that job. It also
skips jobs with zero or one step and reusable-workflow calls. Conditional and
matrix steps count; composite action internals are not expanded, so only the
caller's timeout is considered.

This sum check keeps the job-level fail-fast deadline below the full
end-to-end budget implied by its top-level steps; it compares their literal
timeout bounds and does not estimate concurrent runtime budgets. A parallel
group whose outer step has no supported literal timeout makes the job's sum
unprovable, so the comparison skips that job. When both `rejectStepExceedingJob` and
`rejectJobAtOrAboveStepSum` are enabled, the job timeout must be at least the
longest step timeout and strictly less than the sum of all step timeouts.
Neither `maxMinutes` nor `allow` entries bypass the sum check.

## Options

`include` defaults to workflow `.yml` and `.yaml` files. `maxMinutes` is an
optional cap. `rejectStepExceedingJob` and `rejectJobAtOrAboveStepSum` both
default to `false`. Each `allow` entry names a `path#jobId` and may set a higher
`maxMinutes` cap.

```yaml
rules:
  - rule: github-actions-job-timeouts
    scope: repository
    options:
      include: [".github/workflows/**/*.yml"]
      maxMinutes: 10
      rejectStepExceedingJob: true
      rejectJobAtOrAboveStepSum: true
```

An `allow` entry can raise the cap for one job. It does not skip the sum check:

```yaml
rules:
  - rule: github-actions-job-timeouts
    scope: repository
    options:
      allow:
        - job: ".github/workflows/ci.yml#coverage"
          maxMinutes: 20
```

`timeout-minutes` must be a number or numeric string. Invalid YAML is a
diagnostic, and unused `allow` entries for jobs in scanned files are findings.

## Valid example

With a cap of 10, these two step bounds sum to 13. The job timeout of 10 is at
least the longest step timeout (8) and below the sum, so it satisfies the
configured checks and cap.

```yaml
jobs:
  test:
    runs-on: ubuntu-slim
    timeout-minutes: 10
    steps:
      - run: echo again
        timeout-minutes: 5
      - run: echo last
        timeout-minutes: 8
```

## Counterexample

This job fails the sum check because its job timeout equals the two step bounds
combined (5 + 8 = 13).

```yaml
jobs:
  test:
    runs-on: ubuntu-slim
    timeout-minutes: 13
    steps:
      - run: echo again
        timeout-minutes: 5
      - run: echo last
        timeout-minutes: 8
```

With both timeout relationship checks enabled, set a literal job timeout that
is at least the longest step and strictly below the sum. Under the example cap
of 10, valid values are 8 through 10. Use an `allow` entry with a higher cap
only when the job legitimately needs to exceed that cap; it does not change
the sum requirement.

## Related rules

[`github-actions-test-timeout-literals`](github-actions-test-timeout-literals.md)
keeps tests from duplicating this policy, while
[`github-actions-action-timeout-pair`](github-actions-action-timeout-pair.md)
covers nested action limits.

## Suppression

Use `no-mistakes-disable-next-line github-actions-job-timeouts` for a one-job
exception or `no-mistakes-disable-file` for an exceptional workflow. Prefer a
reasoned `allow` entry when only a known job needs a higher cap.
