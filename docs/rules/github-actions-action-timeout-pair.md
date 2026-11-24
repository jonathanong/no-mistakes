# `github-actions-action-timeout-pair`

## Why and when

Use this rule when an action's outer workflow timeout and its native timeout
must fail together. It prevents a local wrapper or third-party action from
outliving the step that owns it, and lets an action's own deadline trigger
cleanup before GitHub cancels the step.

## Configuration

```yaml
rules:
  - rule: github-actions-action-timeout-pair
    scope: repository
    options:
      uses:
        - ./.github/actions/setup-aws
        - aws-actions/configure-aws-credentials@
      stepTimeoutMinutes: 2
      nestedInput: action-timeout-s
      nestedTimeoutSeconds: 90
      nestedBelowStepTimeout: true
      forbidNestedInComposite: true
```

## What it requires

Matching workflow `uses:` steps must satisfy any configured exact
`stepTimeoutMinutes` with a literal step-level `timeout-minutes`. Direct
third-party matches (entries ending in `@`, not local `./` paths) must also
satisfy any configured exact `nestedInput` and `nestedTimeoutSeconds` with a
bare numeric literal.

Composite-action steps cannot express `timeout-minutes`, so matching calls
belong in the workflow or inside a configured local wrapper that forwards the
nested input. With `forbidNestedInComposite: true` (the default), matching
`uses:` calls inside any composite that is not itself the configured local
wrapper path are forbidden. Direct third-party calls inside the local wrapper
still require any configured exact `nestedInput` and `nestedTimeoutSeconds`.

With `nestedBelowStepTimeout: true`, a matching direct third-party workflow
step's finite, nonnegative `with[nestedInput]` number in seconds must be
strictly less than its literal step-level `timeout-minutes` multiplied by 60.
Only step-level `timeout-minutes` is checked; a job-level `timeout-minutes` is
not considered. Both exact-value options (`stepTimeoutMinutes` and
`nestedTimeoutSeconds`) can be omitted when only the relative comparison is
needed; when set, exact-value findings are always emitted regardless of
`nestedBelowStepTimeout` and `unanalyzable`.

## Valid example

Every matching caller step has a literal `timeout-minutes`, and the direct
third-party action receives a bare numeric nested timeout strictly below the
120-second step cap.

```yaml
jobs:
  deploy:
    runs-on: ubuntu-slim
    steps:
      - uses: ./.github/actions/setup-aws
        timeout-minutes: 2
      - uses: aws-actions/configure-aws-credentials@v4
        timeout-minutes: 2
        with:
          action-timeout-s: 90
```

## Counterexample

A workflow step calls the local wrapper without `timeout-minutes`, or a
third-party call quotes the nested timeout number.

```yaml
jobs:
  deploy:
    runs-on: ubuntu-slim
    steps:
      - uses: ./.github/actions/setup-aws
      - uses: aws-actions/configure-aws-credentials@v4
        timeout-minutes: 2
        with:
          action-timeout-s: "90"
```

With `nestedBelowStepTimeout: true`, a 150-second deadline exceeds the
120-second step cap (and a 120-second deadline also fails because the deadlines
are equal).

```yaml
jobs:
  deploy:
    runs-on: ubuntu-slim
    steps:
      - uses: aws-actions/configure-aws-credentials@v4
        timeout-minutes: 2
        with:
          action-timeout-s: 150
```

## Fix

Set a literal step-level `timeout-minutes` on every matching workflow step,
pass the nested timeout as a bare number strictly below `timeout-minutes * 60`
(consistent with any configured exact values), and keep matching `uses:` calls
out of unrelated composite actions.

```yaml
jobs:
  deploy:
    runs-on: ubuntu-slim
    steps:
      - uses: ./.github/actions/setup-aws
        timeout-minutes: 2
      - uses: aws-actions/configure-aws-credentials@v4
        timeout-minutes: 2
        with:
          action-timeout-s: 90
```

## Suppression

Place `# no-mistakes-disable-next-line github-actions-action-timeout-pair`
immediately above the reported step key, or use
`no-mistakes-disable-file github-actions-action-timeout-pair` only for an
intentional file-wide wrapper exception. Prefer narrowing `uses` when an action
is outside the policy.

## Options and defaults

- `uses` accepts exact matches and case-insensitive prefixes ending in `@`.
  Entries are trimmed and a trailing slash is stripped. An empty list reports
  no findings. Local `./` entries identify local wrappers; `@` prefix entries
  identify direct third-party actions subject to nested input checks.
- `stepTimeoutMinutes` and `nestedTimeoutSeconds` set optional exact integer
  values. `nestedInput` names the third-party action's `with` key.
- `nestedBelowStepTimeout` defaults to `false`. Enable it with `nestedInput`
  to compare the declared deadlines on direct third-party workflow steps. A
  job-level `timeout-minutes` is not considered.
- `include` defaults to `.github/workflows/**/*.yml` and
  `.github/workflows/**/*.yaml`, plus `.github/actions/**/action.yml` and
  `.github/actions/**/action.yaml`.
- `forbidNestedInComposite` defaults to `true`. Matching calls are forbidden
  in composites outside the configured local wrapper. Composite steps have
  no caller-side `timeout-minutes`, so the deadline comparison applies only
  to direct workflow calls.

## Analysis

### Effort

Runs at `max` only. Configuring `effort: low` (on the rule, or as a repository
or CLI default) is not an error: `check` records the warning
`requested effort low; effective effort max because this rule has no low implementation`
and runs the rule at `max`.

### Unanalyzable inputs

With `nestedBelowStepTimeout: true`, `${{ }}` expressions, quoted numbers,
missing values, negative numbers, and other non-literal operands in the
below-the-cap comparison are handled by `options.unanalyzable`. Use `report`
for a finding (the default), `advise` for an advisory, or `ignore` to skip the
comparison's uncertainty. The comparison reports a numeric deadline at or above
the step cap under all three policies, and exact-value checks
(`stepTimeoutMinutes` and `nestedTimeoutSeconds`) still emit findings
regardless of `options.unanalyzable`.

Invalid YAML is skipped so [`github-actions-job-timeouts`](github-actions-job-timeouts.md)
can report it; it is not sent through `options.unanalyzable`. Invalid UTF-8 is
decoded lossily and scanned, so a violation next to a stray byte is still
reported. A file that cannot be read for any other reason is skipped and
produces no finding.

## Related rules

[`github-actions-job-timeouts`](github-actions-job-timeouts.md) sets the job
ceiling. [`github-actions-composite-step-schema`](github-actions-composite-step-schema.md)
keeps workflow-only timeout keys out of composite steps.
[`github-actions-action-input-policy`](github-actions-action-input-policy.md)
requires or bans `with` inputs for configured action families.
