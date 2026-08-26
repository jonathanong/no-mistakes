# `github-actions-action-timeout-pair`

## Why and when

Use this rule when an action's outer workflow timeout and its native timeout
must fail together. It prevents a wrapper or third-party action from outliving
the step that owns it.

## What it requires

Matching workflow `uses:` steps need a literal step timeout. Direct third-party
matches also need the configured numeric nested input; matching calls inside
unrelated composite actions are forbidden by default.

## Valid example

The fixed workflow below is valid: every matching caller step has a literal
`timeout-minutes`, and the third-party action receives a numeric nested timeout.

## Suppression

Use `no-mistakes-disable-next-line github-actions-action-timeout-pair` on the
specific workflow step, or a file directive only for an intentional wrapper
exception. Prefer narrowing `uses` when the action is outside the policy.

## Options and defaults

`uses` is the required exact/prefix match list. `stepTimeoutMinutes`,
`nestedInput`, and `nestedTimeoutSeconds` define the paired values; `include`
defaults to workflow and composite `.yml`/`.yaml` locations, and
`forbidNestedInComposite` defaults to `true`.

## Related rules

[`github-actions-job-timeouts`](github-actions-job-timeouts.md) sets the job
ceiling, while [`github-actions-composite-step-schema`](github-actions-composite-step-schema.md)
keeps workflow-only timeout keys out of composite steps.

Require a caller-side literal `timeout-minutes` and, for direct third-party
`uses:`, a nested native timeout input. Composite-action steps cannot express
`timeout-minutes`, so matching calls belong in the workflow (or inside a
configured local wrapper that forwards the nested input).

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
      forbidNestedInComposite: true
```

`uses` entries match after trim and a trailing-slash strip. An entry that
ends in `@` is a case-insensitive prefix; other entries are exact. Direct
third-party calls are the prefix `@` form, not a local `./` path: those
steps must also set `with[nestedInput]` to the number `nestedTimeoutSeconds`
(not a quoted string).

`include` defaults to `.github/workflows/**/*.yml` / `*.yaml` and
`.github/actions/**/action.yml` / `action.yaml`. Empty `uses` produces no
findings. Invalid YAML is skipped so `github-actions-job-timeouts` can report
it. `forbidNestedInComposite` (default true) rejects matching `uses:` inside
any composite that is not itself the local wrapper path.

Counterexample: a workflow step calls the local wrapper without
`timeout-minutes`, or a third-party call omits the nested timeout number.

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

Fix: set a literal `timeout-minutes` on the workflow step, and pass the
nested timeout as a bare number. Keep matching `uses:` out of other
composites.

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

Use `no-mistakes-disable-next-line github-actions-action-timeout-pair` or
`no-mistakes-disable-file` for a one-off exception.
