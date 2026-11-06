# Workflow activity type policy

Use the [`workflow-topology-policy`](../rules/workflow-topology-policy.md)
option `forbiddenActivityTypes` to ban explicit GitHub Actions trigger activity
types. This keeps activity types that create unwanted workflow runs out of the
repository's trigger declarations.

```yaml
rules:
  - rule: workflow-topology-policy
    scope: repository
    message: Remove the forbidden activity type from this trigger.
    options:
      forbiddenActivityTypes:
        - types: [edited]
          events: [pull_request, pull_request_target]
        - types: [deleted]
```

Each entry has a `types` list of exact, case-sensitive strings and an optional
`events` list of exact event names. Omitted `events` applies to every event;
`events: []` applies to none. Omitted `forbiddenActivityTypes`, an empty policy
list, or an entry with `types: []` bans nothing.

The policy inspects `on.<event>.types` in mapping-form triggers. It matches a
string scalar or string members of a sequence. For the configuration above,
this declaration is a counterexample:

```yaml
on:
  pull_request:
    types: [opened, edited]
```

Remove `edited` while retaining the intended allowed activity types:

```yaml
on:
  pull_request:
    types: [opened]
```

Findings point to the `types` key, including when a matched member is on a
later line. The standard rule-level `message` overrides the diagnostic text.
Use a line or next-line directive at that key for a deliberate exception:

```yaml
on:
  pull_request:
    # no-mistakes-disable-next-line workflow-topology-policy
    types: [opened, edited]
```

`# no-mistakes-disable-file workflow-topology-policy` suppresses the whole
workflow. Prefer removing unwanted activity types to suppressing the policy.

Scalar or sequence `on` declarations, event mappings without `types`, and
non-string `types` values or sequence members are ignored. The policy judges
explicit declarations; it does not expand default activity types or validate
which activity types GitHub accepts for an event. No automatic fix is applied.

CLI `check`, async Node `check({ root, config })`, and `analyzeProject()` check
reports read this same YAML option and return the same findings and suppressions.
