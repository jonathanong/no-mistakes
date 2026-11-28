# Workflow secrets inherit

Configure `workflow-topology-policy.options.forbidSecretsInherit: true` to
prohibit `secrets: inherit` on reusable workflow calls. `false` and an omitted
option report nothing for this option. The match is the exact string
`inherit`, including a quoted `"inherit"`. `Inherit` does not match.

```yaml
rules:
  - rule: workflow-topology-policy
    scope: repository
    options:
      forbidSecretsInherit: true
```

Use `no-mistakes check --root <repo> --format json`. Local and remote
`uses` calls are both checked. A missing `secrets` key and an explicit mapping,
including `secrets: {}`, pass. A job without a reusable-workflow `uses` is
ignored.

Map each secret by name. Findings point to the `secrets` key, so put line and
next-line suppressions there. Standard rule-level `message` and
`no-mistakes-disable-file workflow-topology-policy` also apply. Async
`check({ root, config })` and `analyzeProject()` check reports use the same
configuration and findings. Secret inventory still treats inherit as a passed
caller secret unless this ban is on.

`requireExplicitWorkflowCallBindings: true` also reports `secrets: inherit`
when `forbidSecretsInherit` is off, and it requires every secret declared on
a local callee plus every required input to be mapped. When both options are
on, that inherit finding is emitted once. When at least one of those options is on, a remote call that sets
`secrets: inherit` reports that finding and returns without an unanalyzable
diagnostic, so `options.unanalyzable` does not cover unchecked remote inputs
on that call. Other unverifiable calls follow `options.unanalyzable`:
`report`, `advise` (the default), or `ignore`. `ignore` omits those
diagnostics. `requireExplicitWorkflowCallBindings: false` reports nothing
for that option.

See the [configuration reference](https://github.com/jongleberry/no-mistakes/blob/main/docs/configuration/workflow-topology-policy-secrets-inherit.md)
and [rule reference](https://github.com/jongleberry/no-mistakes/blob/main/docs/rules/workflow-topology-policy.md).
