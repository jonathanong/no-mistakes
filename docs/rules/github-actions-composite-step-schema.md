# `github-actions-composite-step-schema`

## Why and when

Use this rule when composite actions are maintained locally. GitHub silently
ignores workflow-only keys in a composite step, so a syntactically valid action
can otherwise omit the intended runtime limit.

## What it catches

It rejects unsupported keys in `runs.steps` for `runs.using: composite`; Docker
and Node actions are outside the rule.

## Options

`include` defaults to `.github/actions/**/action.{yml,yaml}`. An empty
`allowedKeys` uses GitHub's built-in composite-step allowlist; `extraForbiddenKeys`
adds explicit denials. No other rule-specific options exist.

## Valid example

The `Clean Action` example below is valid because each composite step uses only
the documented step keys.

## Suppression

Use a file directive for an intentionally exceptional action, or a line
directive on the unsupported key when the finding has that key's location.

## Related rules

[`github-actions-action-timeout-pair`](github-actions-action-timeout-pair.md)
places timeouts on the caller step; [`github-actions-job-timeouts`](github-actions-job-timeouts.md)
sets the containing job limit.

Validate composite-action `runs.steps` against the GitHub composite-step
schema. GitHub documents a fixed set of step keys for `runs.using: composite`
actions. Workflow-only keys such as `timeout-minutes` are silently ignored at
runtime and should be set on the calling workflow step instead.

actionlint does not cover this schema gap.

```yaml
rules:
  - rule: github-actions-composite-step-schema
    scope: repository
    options:
      include:
        - ".github/actions/**/action.yml"
        - ".github/actions/**/action.yaml"
      allowedKeys: []  # if empty, use defaults
      extraForbiddenKeys: []  # optional extra deny list
```

By default the rule only inspects `action.yml` / `action.yaml` files under
`.github/actions/`. Override `include` to scan additional action metadata
files. When `allowedKeys` is empty, the default GitHub allowlist is used:
`name`, `id`, `if`, `uses`, `run`, `shell`, `with`, `env`,
`working-directory`, and `continue-on-error`. `extraForbiddenKeys` can ban a
key that is otherwise allowed.

YAML is parsed with `serde_yaml`. Description prose that mentions
`timeout-minutes` does not flag. Invalid YAML emits a diagnostic instead of
being skipped. Non-composite actions (`runs.using: docker` / `node20` / …)
are ignored even if they contain `timeout-minutes`.

**Pass:**

```yaml
name: Clean Action
runs:
  using: composite
  steps:
    - name: Setup
      uses: actions/checkout@main
    - name: Run script
      run: pnpm install
      shell: bash
```

**Counterexample:**

```yaml
name: Timeout Action
runs:
  using: composite
  steps:
    - name: Do something
      timeout-minutes: 5
      run: echo hello
      shell: bash
```

**Fix:** Remove the unsupported key from the composite step and set it on the
calling workflow step instead:

```yaml
# .github/workflows/ci.yml
- uses: ./.github/actions/my-action
  timeout-minutes: 5
```

Use `# no-mistakes-disable-file github-actions-composite-step-schema` to
suppress an entire action file. Line-level
`# no-mistakes-disable-next-line` / `# no-mistakes-disable-line` comments
also apply to the flagged key line. A `timeout-minutes:` mention inside a
YAML `|` / `>` block scalar is documentation, not a step key.

**Scope:** Checks `.github/actions/**/action.{yml,yaml}` by default.
