# `github-actions-action-timeout-pair`

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
