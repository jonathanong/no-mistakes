# Workflow activity types

Configure `workflow-topology-policy.options.forbiddenActivityTypes` to prevent
explicit trigger activity types from creating unwanted workflow runs.

```yaml
rules:
  - rule: workflow-topology-policy
    scope: repository
    options:
      forbiddenActivityTypes:
        - types: [edited]
          events: [pull_request, pull_request_target]
```

Use `no-mistakes check --root <repo> --format json`. Matching is exact and case
sensitive. Omitted `events` applies to all events; `events: []` applies to none.
Only mapping-form `on.<event>.types` string scalars and string sequence members
are checked. Scalar/list `on`, missing `types`, and non-string values are ignored;
default activity types are not expanded.

Remove the forbidden type and retain intended allowed types. Findings point to
the `types` key, so put line/next-line suppressions there. Standard rule-level
`message` and `no-mistakes-disable-file workflow-topology-policy` also apply.
Async `check({ root, config })` and `analyzeProject()` check reports use the same
configuration and findings.

See the [configuration reference](https://github.com/jonathanong/no-mistakes/blob/main/docs/configuration/workflow-topology-policy-activity-types.md)
and [rule reference](https://github.com/jonathanong/no-mistakes/blob/main/docs/rules/workflow-topology-policy.md).
