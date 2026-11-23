# `github-actions-action-timeout-pair`

An action can use its own deadline to stop work and start cleanup. Set that
deadline below the workflow step's `timeout-minutes` so the action can act
before GitHub cancels the step.

## Configuration

```yaml
rules:
  - rule: github-actions-action-timeout-pair
    scope: repository
    options:
      uses:
        - example-org/publish-widget@
      nestedInput: deadline-seconds
      nestedBelowStepTimeout: true
```

With `nestedBelowStepTimeout: true`, a matching third-party workflow step's
finite, nonnegative `with[nestedInput]` number in seconds must be strictly less
than its literal `timeout-minutes` multiplied by 60. The comparison uses the values in the
workflow and does not require either exact-value option.

## Valid example

The step has a 120-second cap and the action's own deadline is 90 seconds.

```yaml
jobs:
  publish:
    runs-on: ubuntu-slim
    steps:
      - uses: example-org/publish-widget@aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa
        timeout-minutes: 2
        with:
          deadline-seconds: 90
```

## Counterexample

A 150-second deadline exceeds the 120-second cap. A 120-second deadline also
fails because the deadlines are equal.

```yaml
jobs:
  publish:
    runs-on: ubuntu-slim
    steps:
      - uses: example-org/publish-widget@aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa
        timeout-minutes: 2
        with:
          deadline-seconds: 150
```

## Fix

Lower `deadline-seconds` to a bare number below 120, such as `90`, so the
action has time to start cleanup. Keep both values consistent with any
configured exact values.

## Suppression

Place `# no-mistakes-disable-next-line github-actions-action-timeout-pair`
immediately above the reported key. Use
`no-mistakes-disable-file github-actions-action-timeout-pair` for a file-wide
exception.

## Options and defaults

- `uses` accepts exact matches and case-insensitive prefixes ending in `@`.
  Entries are trimmed and a trailing slash is stripped. An empty list reports
  no findings. Third-party matches for nested input checks use the `@` prefix
  form.
- `stepTimeoutMinutes` and `nestedTimeoutSeconds` set optional exact integer
  values. `nestedInput` names the third-party action's `with` key.
- `nestedBelowStepTimeout` defaults to `false`. Enable it with `nestedInput`
  to compare the declared deadlines on direct third-party workflow steps.
- `include` defaults to `.github/workflows/**/*.yml` and
  `.github/workflows/**/*.yaml`, plus `action.yml` and `action.yaml` under
  `.github/actions/`.
- `forbidNestedInComposite` defaults to `true`. Matching calls are forbidden
  in composites outside the configured local wrapper. Composite steps have
  no caller-side `timeout-minutes`, so the deadline comparison applies only
  to direct workflow calls.

## Analysis

### Effort

Runs at `max`. A request for `low` records a warning and runs the rule at
`max`.

### Unanalyzable inputs

With the comparison enabled, expressions, quoted numbers, and missing or
otherwise unsupported operands follow `options.unanalyzable`. Use `report` for a finding (the
default), `advise` for an advisory, or `ignore` to skip the uncertainty.
The comparison reports a numeric deadline at or above the step cap under
all three policies.

Invalid YAML is skipped. Invalid UTF-8 is decoded lossily and scanned. Files
that cannot be read for another reason are skipped.

## Related rules

[`github-actions-job-timeouts`](github-actions-job-timeouts.md) sets the job
ceiling. [`github-actions-composite-step-schema`](github-actions-composite-step-schema.md)
keeps workflow-only timeout keys out of composite steps.
[`github-actions-action-input-policy`](github-actions-action-input-policy.md)
requires or bans `with` inputs for configured action families.
