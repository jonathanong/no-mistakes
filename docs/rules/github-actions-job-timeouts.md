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

## Effort

`effort: max` is the default and keeps every job and step comparison above.
`effort: low` uses enabled, unsuppressed
[`github-actions-step-timeouts`](github-actions-step-timeouts.md) applications
as partners. Together, they must cover this rule's workflow scope and have no
`allow` entries. Each partner's `maxMinutes` (360 by default) certifies a
literal upper bound for its executable steps. If a job has a supported literal
timeout at least as large as that bound, this rule skips its redundant
`rejectStepExceedingJob` comparison for executable steps. The step rule still
reports a missing, malformed, or over-cap step timeout. For a job timeout below
the certified bound, this rule keeps the full step comparison. It also keeps the
comparison for step mappings the partner does not inspect, such as
metadata-only steps.

Low effort keeps the job timeout requirement, `maxMinutes` cap, stale job
`allow` findings, and `rejectJobAtOrAboveStepSum` check. Missing, disabled,
suppressed, exception-bearing, or insufficiently scoped partners cannot certify
the steps: the check warns and falls back to `max`. When several partner
applications cover the workflows, a narrower cap from one application does
not certify steps that another application permits above it; the largest
certified cap applies. See
[configuration](../configuration/rules.md#rule-applications) for the shared
effort precedence and [check output](../cli/check.md) for fallback reporting.

For example, this configuration caps steps at five minutes and jobs at ten:

```yaml
rules:
  - rule: github-actions-step-timeouts
    scope: repository
    options:
      include: [".github/workflows/**/*.yml"]
      maxMinutes: 5
  - rule: github-actions-job-timeouts
    scope: repository
    options:
      include: [".github/workflows/**/*.yml"]
      effort: low
      maxMinutes: 10
      rejectStepExceedingJob: true
```

With that policy, a job timeout of ten minutes and two valid step timeouts of
five minutes each pass both rules:

```yaml
jobs:
  test:
    runs-on: ubuntu-slim
    timeout-minutes: 10
    steps:
      - run: echo first
        timeout-minutes: 5
      - run: echo second
        timeout-minutes: 5
```

This over-cap step is reported by the step rule, even though this job rule can
skip its executable-step comparison at low effort:

```yaml
jobs:
  test:
    runs-on: ubuntu-slim
    timeout-minutes: 10
    steps:
      - run: echo too-long
        timeout-minutes: 12
```

Changing the job timeout to four minutes instead retains the job rule's full
comparison: a five-minute step exceeds that job even though it satisfies the
partner's cap. At `max`, the job rule also reports a step exceeding a job's
timeout. A low-effort report can therefore have fewer *job-rule* findings;
the partner reports over-cap steps.

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

## Fix

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
