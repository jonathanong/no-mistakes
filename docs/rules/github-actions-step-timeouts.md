# `github-actions-step-timeouts`

## Why and when

Use this rule when a job timeout is too broad to expose a stalled checkout,
download, or command promptly. GitHub supports `timeout-minutes` on workflow
`run` and `uses` steps, with a 360-minute maximum.

## What it requires

Every authored executable step under `jobs.<job_id>.steps` must set a positive,
literal YAML number for `timeout-minutes`. GitHub Actions rejects quoted
numbers. Missing values, zero, negative values, booleans, expressions, and
values above the cap are findings.
The rule scans `.github/workflows/**/*.yml` and `*.yaml` by default. It skips
jobs that call reusable workflows, which have no `steps`, and it does not scan
composite action metadata, where step `timeout-minutes` is unsupported.
Executable children of a workflow `parallel` group are checked recursively.
Metadata-only steps without `run` or `uses` are outside the rule's scope.

## Options

`include` replaces the default workflow globs. `maxMinutes` sets an optional
cap below GitHub's 360-minute maximum. The effective cap never exceeds 360.
`allow` accepts a reasoned exception for a specific step, identified as
`path#jobId#stepId`. For a step without `id`, use its 1-based position in the
job's `steps` array, such as `path#jobId#2`. An allow entry permits an absent or
expression timeout; malformed literal values still fail. Its optional
`maxMinutes` overrides the ordinary cap for a literal timeout, but cannot
exceed 360. Blank reasons, duplicate entries, and stale entries in scanned
workflows are findings. A raising entry is stale if its step already has a valid
literal timeout within the ordinary cap; a lower cap stays active as a stricter
limit. For a step inside `parallel`, append
`/childId` or a 1-based child position to its group's selector; for example,
`.github/workflows/ci.yml#build#2/frontend` or
`.github/workflows/ci.yml#build#2/1`.

```yaml
rules:
  - rule: github-actions-step-timeouts
    scope: repository
    options:
      maxMinutes: 15
      allow:
        - step: ".github/workflows/ci.yml#build#download"
          reason: "The input matrix selects a bounded timeout."
```

## Valid example

```yaml
jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
        timeout-minutes: 2
      - run: pnpm test
        timeout-minutes: 10
```

## Counterexample and fix

The checkout below can hang until the job deadline:

```yaml
steps:
  - uses: actions/checkout@v4
```

Set a literal timeout on the step:

```yaml
steps:
  - uses: actions/checkout@v4
    timeout-minutes: 2
```

## Suppression

Use `no-mistakes-disable-next-line github-actions-step-timeouts` above a step
or `no-mistakes-disable-file github-actions-step-timeouts` at the top of a
workflow. Prefer a reasoned `allow` entry when an expression is intentional.

## Related rules

[`github-actions-job-timeouts`](github-actions-job-timeouts.md) bounds the
containing job separately.
