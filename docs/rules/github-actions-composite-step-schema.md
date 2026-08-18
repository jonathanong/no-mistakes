# `github-actions-composite-step-schema`

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
suppress an entire action file.

**Scope:** Checks `.github/actions/**/action.{yml,yaml}` by default.
