# Workflow secrets inherit policy

Use the [`workflow-topology-policy`](../rules/workflow-topology-policy.md)
option `forbidSecretsInherit` to prohibit `secrets: inherit` on reusable
workflow calls. Callers then pass each secret by name instead of forwarding
every available secret.

```yaml
rules:
  - rule: workflow-topology-policy
    scope: repository
    message: Map each secret explicitly.
    options:
      forbidSecretsInherit: true
```

`false` is the default and reports nothing. The match is the exact string
`inherit`. `Inherit` does not match. A quoted `"inherit"` does.

This caller is a counterexample:

```yaml
jobs:
  ship:
    uses: ./.github/workflows/orders.yml
    secrets: inherit
```

Pass the declared secret explicitly:

```yaml
jobs:
  ship:
    uses: ./.github/workflows/orders.yml
    secrets:
      ORDER_TOKEN: ${{ secrets.ORDER_TOKEN }}
```

A call with no `secrets` key, and a call with `secrets: {}`, pass no secrets
and are quiet. The ban applies to local `./` calls and to remote
`owner/repo/.github/workflows/name.yml@ref` calls. A job that does not call a
reusable workflow is ignored, even if it contains the text `secrets: inherit`.

Findings point to the `secrets` key. The standard rule-level `message`
overrides the diagnostic text. Use a line or next-line directive at that key
for a deliberate exception:

```yaml
jobs:
  ship:
    uses: ./.github/workflows/orders.yml
    # no-mistakes-disable-next-line workflow-topology-policy
    secrets: inherit
```

`# no-mistakes-disable-file workflow-topology-policy` suppresses the whole
workflow. Prefer an explicit mapping to suppressing the policy.

This option does not change secret-inventory completeness. With the ban off,
`secrets: inherit` still passes caller secrets through to the inventory check.
No automatic fix is applied.

CLI `check`, async Node `check({ root, config })`, and `analyzeProject()` check
reports read this same YAML option and return the same findings and suppressions.

## `requireExplicitWorkflowCallBindings`

Set `requireExplicitWorkflowCallBindings: true` to require reusable-workflow
callers to pass every declared secret explicitly (both required and optional
secrets under `on.workflow_call.secrets`) and every `required: true` input under
`on.workflow_call.inputs`:

```yaml
rules:
  - rule: workflow-topology-policy
    scope: repository
    options:
      requireExplicitWorkflowCallBindings: true
      unanalyzable: advise # report | advise | ignore
```

`false` is the default and reports nothing. When `true`:

- `secrets: inherit` is reported at the `secrets` key (deduplicated when
  `forbidSecretsInherit: true` is also enabled).
- Every secret declared in a local callee's `on.workflow_call.secrets` (whether
  `required: true` or optional) must be mapped to a non-null scalar under
  `secrets:` (matched case-insensitively, following GitHub Actions).
- Every input declared with `required: true` in a local callee's
  `on.workflow_call.inputs` must be passed as a non-null scalar under `with:`.
  Optional inputs may be omitted.
- Findings point to `secrets:` or `with:` when present (or to the caller job key
  when the block is omitted), and to the individual entry key when left empty.
- Unverifiable inputs — remote `uses:` calls, missing or non-callable local
  callees, malformed workflow YAML, non-mapping `secrets:` / `with:` values
  (such as `${{ ... }}`), non-scalar entry values, and case-colliding keys — are
  routed through `options.unanalyzable` (default `advise`, never a silent pass).
- Non-boolean values for `requireExplicitWorkflowCallBindings` fail at config
  load with exit code 2.
- Standard `no-mistakes-disable-line`, `no-mistakes-disable-next-line`, and
  `no-mistakes-disable-file` directives for `workflow-topology-policy` suppress
  both findings and advisories.
