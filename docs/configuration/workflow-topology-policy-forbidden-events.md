# Workflow forbidden events

Use the [`workflow-topology-policy`](../rules/workflow-topology-policy.md)
option `forbiddenEvents` to ban named GitHub Actions trigger events. This keeps
events that create unwanted workflow runs out of `on`.

```yaml
rules:
  - rule: workflow-topology-policy
    scope: repository
    message: Remove the forbidden trigger event.
    options:
      forbiddenEvents: [pull_request_target]
```

`forbiddenEvents` is a list of exact, case-sensitive event names. An omitted
option or `forbiddenEvents: []` bans nothing. `Pull_Request_Target` does not
match `pull_request_target`.

The policy reads scalar, sequence, and mapping `on` forms. For the
configuration above, these declarations are counterexamples:

```yaml
on: pull_request_target
```

```yaml
on: [push, pull_request_target]
```

```yaml
on:
  pull_request_target:
    types: [opened]
```

Use `pull_request` in the same form:

```yaml
on: pull_request
```

```yaml
on: [push, pull_request]
```

```yaml
on:
  pull_request:
    types: [opened]
```

A mapping entry with a value, including `types`, is reported on that event
key. A scalar, a sequence, and a mapping entry whose value is null
(`pull_request_target:`) are reported on the `on` key, because those forms do
not store a child configuration object. Duplicate event names in one `on`
produce one finding. Non-string sequence members are ignored.

The standard rule-level `message` overrides the diagnostic text. Use a line or
next-line directive at the reported key for a deliberate exception:

```yaml
on: pull_request_target # no-mistakes-disable-line workflow-topology-policy
```

```yaml
on:
  # no-mistakes-disable-next-line workflow-topology-policy
  pull_request_target:
    types: [opened]
```

`# no-mistakes-disable-file workflow-topology-policy` suppresses the whole
workflow. Prefer removing the unwanted event to suppressing the policy.

The policy does not expand default activity types and does not validate which
events GitHub accepts. No automatic fix is applied.

CLI `check`, async Node `check({ root, config })`, and `analyzeProject()` check
reports read this same YAML option and return the same findings and suppressions.
