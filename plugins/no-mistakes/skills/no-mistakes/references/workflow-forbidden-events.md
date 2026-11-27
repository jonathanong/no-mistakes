# Workflow forbidden events

Configure `workflow-topology-policy.options.forbiddenEvents` to ban named
trigger events. Matching is exact and case sensitive. An omitted list or `[]`
bans nothing.

```yaml
rules:
  - rule: workflow-topology-policy
    scope: repository
    options:
      forbiddenEvents: [pull_request_target]
```

Use `no-mistakes check --root <repo> --format json`. Scalar `on`, sequence
`on`, and mapping `on` are all checked. A mapping entry, including a null
value, is reported on that event key. A sequence item is reported on the
item's line. A scalar is reported on the `on` key, and a flow sequence item
shares that line. Put line and next-line suppressions on the reported line.
Duplicate names in one workflow produce one finding.

Replace the banned event with the intended event, such as `pull_request`.
Standard rule-level `message` and `no-mistakes-disable-file
workflow-topology-policy` also apply. Async `check({ root, config })` and
`analyzeProject()` check reports use the same configuration and findings.

See the [configuration reference](https://github.com/jongleberry/no-mistakes/blob/main/docs/configuration/workflow-topology-policy-forbidden-events.md)
and [rule reference](https://github.com/jongleberry/no-mistakes/blob/main/docs/rules/workflow-topology-policy.md).
