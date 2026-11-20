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

A mapping entry is reported on that event key, including `types`, a null
value such as `pull_request_target:`, a quoted `"pull_request_target":`,
and a quoted `"on":` key. A comment between `on:` and the event key does
not move the finding. A sequence item is reported on the item's line. A
one-line flow sequence such as `on: [push, pull_request_target]` shares
the `on` line, because the item is written there. An item on its own line
in a multi-line flow sequence is reported on that line. Duplicate event
names in one `on` produce one finding. Non-string sequence members are
ignored. A `no-mistakes-disable-line` directive on the `on:` line of a
null mapping or a block sequence no longer applies, because the finding
now lands on the event key, and that directive is an
`unused-disable-directive` advisory.

A YAML tag or a second document leaves the file without a source map. A
tagged workflow still reports the forbidden event on line 1. A
multi-document file is not one workflow, so its events are not reported.

The standard rule-level `message` overrides the diagnostic text. Use a line or
next-line directive at the reported key for a deliberate exception:

```yaml
on: pull_request_target # no-mistakes-disable-line workflow-topology-policy
```

```yaml
on:
  pull_request_target: # no-mistakes-disable-line workflow-topology-policy
```

```yaml
on:
  - push
  - pull_request_target # no-mistakes-disable-line workflow-topology-policy
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
