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
`equals-file`, `object-shape`, and `ancestor-override-subset`. JSON and JSONC
files (`.json`, `.jsonc`) are parsed with comment support; YAML is used for other
extensions. A file that cannot be parsed is a finding, not a silent skip.

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

`ancestor-override-subset` follows local `extends` chains (string or string
array) relative to the declaring file. Package specifiers are skipped; cycles
and paths that leave the repository root are findings. Parse errors are reported
on the ancestor file. After a nested config becomes the resolution root, ancestor
override `files` globs are matched against inventory children under that nested
directory. An override that matched those children relative to the ancestor
directory but no longer matches relative to the nested directory is lost: its
`rules` must be a value-equal subset of the nested config's top-level `rules`.
Overrides whose globs still match after rebasing, and nested directories with no
inventory children, are ignored. Extra nested rules are allowed. Key names default
to `extends`, `overrides`, `files`, and `rules`, and can be overridden with
`extendsKey`, `overridesKey`, `filesKey`, and `rulesKey`.

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
      - kind: ancestor-override-subset
      - key: plugins
        kind: equals-file
        file: .oxlintrc.json
        fromKey: plugins
```

Counterexample: a config file omits a required key, still contains a banned
legacy key, uses a string where a boolean is required, contains a single-file
entry where a glob is required, has a nested rule object with the wrong
severity, is invalid JSONC/YAML, or extends an ancestor whose path-scoped
override no longer matches after rebasing without restating those rules.

Fix: add the required key, remove the banned key, update the value to match the
configured assertion, make one array entry satisfy `match: any`, restate lost
ancestor override rules on the nested top-level `rules` object, or narrow the
file glob to the configs where the policy applies. Repair parse errors so the
file is valid JSONC or YAML.

Suppression: use `no-mistakes` suppression directives. Findings currently report
line 1 for structured config shape violations, so prefer file-level suppression
for generated config files.

## Why and when

Use this rule when configuration files are an API between tools and need
required keys, forbidden legacy keys, or stable value shapes enforced in CI.
Use `ancestor-override-subset` when nested configs extend a parent and must
keep path-scoped ancestor rules after override globs rebase to the nested
directory.

## What it catches/requires

Each policy applies its key and value assertions to matching JSON, JSONC, or
YAML files. Invalid syntax, wrong scalar shapes, missing required keys, failed
cross-file equality checks, and lost ancestor override rules that are not
restated on the nested config are findings.

## Options and defaults

`policies` is required and defaults to no checks when empty. Each policy may
set `files`, `requiredKeys`, `bannedKeys`, and `valueAssertions`; assertions
support `boolean`, `positive-number`, `string-array`, `record-of-boolean`,
`string-prefix`, `string-glob`, `not-single-file`, `equals`, `equals-file`,
`object-shape`, and `ancestor-override-subset`. Array `match` defaults to `all`;
`when` is optional. `ancestor-override-subset` key names default to `extends`,
`overrides`, `files`, and `rules`.

## Valid example

```yaml
runtime:
  enabled: true
```

With `runtime.enabled` asserted as boolean, this file satisfies the policy.

## Counterexample

```yaml
runtime:
  enabled: "yes"
```

A nested `.oxlintrc.json` that extends a parent with a path-scoped override but
omits those rules after the override glob no longer matches from the nested
directory also fails `ancestor-override-subset`.

## Fix

Repair the file's syntax or value shape, add the required key, remove the
banned legacy key, restate lost ancestor override rules, or narrow the policy
glob to its intended configs.

## Suppression

Use a top-of-file `no-mistakes-disable-file structured-config-policy` directive
for generated config that cannot be edited. Findings currently report line 1.

## Related rules

[`config-path-references`](config-path-references.md) validates referenced
paths; [`no-mistakes-config`](no-mistakes-config.md) checks the analyzer's own
configuration contract.
