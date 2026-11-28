# `github-actions-step-timeouts`

## Why and when

Use this rule when a job timeout is too broad to expose a stalled checkout,
download, or command promptly. GitHub supports `timeout-minutes` on workflow
`run` and `uses` steps, with a 360-minute maximum.

## What it requires

Every authored executable step under `jobs.<job_id>.steps` must set a positive
integer YAML numeric scalar for `timeout-minutes`. This includes blocking `wait` and
`wait-all` steps as well as `run` and `uses`. GitHub Actions rejects quoted
numbers. Missing values, zero, negative values, booleans, expressions, and
values above the cap are findings.
The rule scans `.github/workflows/**/*.yml` and `*.yaml` by default. It skips
jobs that call reusable workflows, which have no `steps`, and it does not scan
composite action metadata, where step `timeout-minutes` is unsupported.
Executable children of a workflow `parallel` group are checked recursively.
Metadata-only steps without `run`, `uses`, `wait`, or `wait-all` are outside the
rule's scope.

## Options

The optional `maxMinutes` must be positive. Each `allow` entry accepts `step`,
`reason`, and an optional `maxMinutes` between 1 and 360. Unknown keys on the
rule options, an allow entry, or an action cap fail at configuration load.

`include` replaces the default workflow globs. `maxMinutes` sets an optional
cap below GitHub's 360-minute maximum. The effective cap never exceeds 360.
`actionCaps` sets per-action-family timeout caps (`{ uses, maxMinutes }`,
`1..=360`) that tighten the cap for matching workflow `uses:` steps (for
example setup actions such as `actions/checkout`, `actions/setup-node`,
`pnpm/action-setup`, or `./.github/actions/setup-pnpm`). Action names match
after stripping `@ref` and trailing slashes, case-insensitively for remote
`owner/repo` actions and verbatim for local `./...` paths. Duplicate families,
dynamic `uses` expressions, or out-of-range caps fail at configuration load.
`onlyConfiguredActions` defaults to `false`; set it to `true` (with a non-empty
`actionCaps`) to require timeouts only on steps matching `actionCaps`.
`actionCaps[].uses` must name one concrete action family. Wildcards such as
`actions/*` fail at configuration load. When `actionCaps` is configured and a
workflow step has a dynamic `uses:` value (`${{ ... }}`), a literal timeout
above `maxMinutes` is a finding even if it is within the smallest action cap.
A missing timeout, or a literal timeout within `maxMinutes` but above that
smallest cap, is an `unanalyzable` diagnostic governed by
`options.unanalyzable` (default `advise`). A literal timeout within both caps
stays quiet.
`allow` accepts a reasoned exception for a specific step, identified as
`path#jobId#stepId`. For a step without `id`, use its 1-based position in the
job's `steps` array, such as `path#jobId#2`. An allow entry permits an absent or
expression timeout; malformed literal values still fail. Its optional
`maxMinutes` overrides the ordinary cap for a literal timeout, but cannot
exceed 360. An entry that omits `maxMinutes` still enforces the global cap.
An entry that suppresses a dynamic-action diagnostic is used even when its
cap equals the ordinary cap. Blank reasons and duplicate entries are findings. Stale entries
are findings only for files this run scanned. A unique allow entry for a
deleted or renamed file is not a stale finding; duplicate entries for that
absent path are still duplicate findings. A raising entry is stale if
its step already has a valid literal timeout within the ordinary cap; a lower
cap stays active as a stricter limit. For a step inside `parallel`, append
`/childId` or a 1-based child position to its group's selector; for example,
`.github/workflows/ci.yml#build#2/frontend` or
`.github/workflows/ci.yml#build#2/1`.

```yaml
rules:
  - rule: github-actions-step-timeouts
    scope: repository
    options:
      maxMinutes: 15
```

An `allow` entry names one step. It does not belong on a workflow that does not
contain that step:

```yaml
rules:
  - rule: github-actions-step-timeouts
    scope: repository
    options:
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

## Counterexample

The checkout below can hang until the job deadline:

```yaml
jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
```

## Fix

Set a literal timeout on the step:

```yaml
steps:
  - uses: actions/checkout@v4
    timeout-minutes: 2
```

## Analysis

Step and job timeout checks share a workflow's parsed YAML during a check.
Each uses its own timeout policy and source locations.

### Effort

- Runs at `max` only. Configuring `effort: low` (on the rule, or as a repository or CLI default) is not an error: `check` records the warning `requested effort low; effective effort max because this rule has no low implementation` and runs the rule at `max`.

### Unanalyzable inputs

Invalid YAML produces an ordinary finding. The rule skips unreadable files,
non-mapping jobs, non-sequence step lists, and non-mapping step entries.
[Workflow and job scanning](https://github.com/jongleberry/no-mistakes/blob/cc505a7c1e9c3cb6ad6d69cce53633b67d0c999e/crates/no-mistakes/src/codebase/rules/github_actions_step_timeouts/scan.rs#L14-L66) and [step scanning](https://github.com/jongleberry/no-mistakes/blob/cc505a7c1e9c3cb6ad6d69cce53633b67d0c999e/crates/no-mistakes/src/codebase/rules/github_actions_step_timeouts/steps.rs#L39-L44).

Following [recall over precision](../../README.md#how-analysis-works), nothing this rule cannot prove is treated as clean.

- When `actionCaps` is configured, a workflow step whose `uses:` contains `${{ }}` (or is non-string) cannot be matched statically to an action family; unless its literal `timeout-minutes` is already within the strictest configured action cap (or covered by an `allow` entry), it emits a diagnostic governed by `options.unanalyzable` (default `advise`).
- Without `actionCaps`, literal `timeout-minutes` values on parsed steps are judged directly and nothing is routed through `options.unanalyzable`.

## Suppression

Use `no-mistakes-disable-next-line github-actions-step-timeouts` above a step
or `no-mistakes-disable-file github-actions-step-timeouts` at the top of a
workflow. Prefer a reasoned `allow` entry when an expression is intentional.

## Related rules

[`github-actions-job-timeouts`](github-actions-job-timeouts.md) bounds the
containing job separately.

Stale allow reporting retains this rule's established diagnostic messages and
severity for checked inputs. Entries outside the checked scope stay silent;
uncertain or partial selector coverage cannot prove staleness. See
[allow-entry hygiene](../configuration/suppressions.md#allow-entry-hygiene).

With `reportUnusedAllowEntries: report` or `advise`, the literal file part of
`allow[].step` also produces `unused-allow-entry` when absent from both the
request inventory and disk within the configured scope. Remove the entry or
fix its path. Existing excluded or ignored files, empty glob matches, and
paths outside a narrowed scope stay silent. See
[allow-entry hygiene](../configuration/suppressions.md#allow-entry-hygiene).

## Invalid UTF-8

Invalid UTF-8 is decoded lossily and scanned, so a violation next to a stray byte is still reported. A file that cannot be read for any other reason is skipped and produces no finding.
