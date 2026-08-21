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
`record-of-boolean`, `string-prefix`, `string-glob`, `not-single-file`, `equals`,
`equals-file`, and `object-shape`. JSON and JSONC files (`.json`, `.jsonc`) are
parsed with comment support; YAML is used for other extensions. A file that cannot
be parsed is a finding, not a silent skip.

Selectors are dotted paths; use numeric parts for array indexes and `[]` to apply
an assertion to every array entry. On `[]` selectors, `match: all` (default)
requires every entry to satisfy the assertion; `match: any` requires at least one
entry in each parent array. Missing parent keys are skipped for `match: any`, so
an override that never mentions a rule is not a failure. A parent key that is
present but not an array still fails `match: any`. `not-single-file` strips a
leading `**/` before looking for glob wildcards, so `**/exact/file.ts` is still
a single-file entry.

`object-shape` accepts `requiredKeys`, `forbiddenKeys`, and `requiredValues`.
`equals-file` compares a key to the same (or `fromKey`) value in another file
relative to the repository root, using the same selectors as other assertions.
The comparison file must stay inside the repository root after normalization;
parse errors are reported on that referenced file. `when` skips the rest of a
policy for a file unless each listed key is a non-empty array or non-empty
string.

```yaml
policies:
  - files: [.oxlintrc.json]
    valueAssertions:
      - key: rules.no-restricted-properties.[]
        kind: object-shape
        match: any
        requiredKeys: [message]
        forbiddenKeys: [object]
        requiredValues:
          property: bind
  - files: ["**/.oxlintrc.json"]
    when:
      - key: extends
    valueAssertions:
      - key: plugins
        kind: equals-file
        file: .oxlintrc.json
        fromKey: plugins
```

Counterexample: a config file omits a required key, still contains a banned
legacy key, uses a string where a boolean is required, contains a single-file
entry where a glob is required, has a nested rule object with the wrong
severity, or is invalid JSONC/YAML.

Fix: add the required key, remove the banned key, update the value to match the
configured assertion, make one array entry satisfy `match: any`, or narrow the
file glob to the configs where the policy applies. Repair parse errors so the
file is valid JSONC or YAML.

Suppression: use `no-mistakes` suppression directives. Findings currently report
line 1 for structured config shape violations, so prefer file-level suppression
for generated config files.
