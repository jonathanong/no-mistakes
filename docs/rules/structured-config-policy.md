# `structured-config-policy`

Requires or bans dotted keys in structured YAML or JSON config files, and can
assert simple value shapes for selected keys.

```yaml
rules:
  - rule: structured-config-policy
    scope: repository
    options:
      policies:
        - files: [app.yml]
          requiredKeys: [runtime.version]
          bannedKeys: [legacy.enabled]
          valueAssertions:
            - key: runtime.enabled
              kind: boolean
            - key: overrides.[].files.[]
              kind: string-prefix
              prefix: "**/"
            - key: overrides.[].files.[]
              kind: not-single-file
            - key: rules.[]
              kind: object-shape
              requiredValues:
                severity: error
```

Supported assertion kinds are `boolean`, `positive-number`, `string-array`,
`string-prefix`, `string-glob`, `not-single-file`, `equals`, and
`object-shape`. Selectors are dotted paths; use numeric parts for array indexes
and `[]` to apply an assertion to every array entry.

Counterexample: a config file omits a required key, still contains a banned
legacy key, uses a string where a boolean is required, contains a single-file
entry where a glob is required, or has a nested rule object with the wrong
severity.

Fix: add the required key, remove the banned key, update the value to match the
configured assertion, or narrow the file glob to the configs where the policy
applies.

Suppression: use `no-mistakes` suppression directives. Findings currently report
line 1 for structured config shape violations, so prefer file-level suppression
for generated config files.
