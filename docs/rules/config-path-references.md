# `config-path-references`

Validates path strings stored inside structured YAML or JSON config files.
`keys` reads dotted string or string-array fields. `presets` extract the
required path fields for well-known configs (not optional `!` / `node_modules`
/ `.git` ignore globs).

```yaml
rules:
  - rule: config-path-references
    scope: repository
    options:
      files: [config/app.yml]
      keys: [paths.requiredFiles, paths.testGlobs]
      baseDir: config-file
      allowGlobs: true
      presets:
        - oxlintrc
        - knip
        - dependabot
        - sgconfig
        - syncpack
        - coverage-rules
```

| Preset | Files | Required paths |
| --- | --- | --- |
| `oxlintrc` | `.oxlintrc.json` / `.oxlintrc.jsonc` (including nested) | relative `jsPlugins[].specifier`, non-glob `overrides.files`, `rules.*.baseline[][0]` |
| `knip` | `knip.json` / `knip.jsonc` | `workspaces.{name}.entry` and `.project`, prefixed by the workspace path unless `.` |
| `dependabot` | `.github/dependabot.yml` | `updates[].directory` |
| `sgconfig` | `sgconfig.yml` | `ruleDirs` |
| `syncpack` | `.syncpackrc.json` | `source` globs |
| `coverage-rules` | `.coverage-rules.yml` | `rules[].paths` |

Counterexample: `config/app.yml` contains `paths.requiredFiles:
["missing.json"]`, and `config/missing.json` does not exist. Or
`.github/dependabot.yml` lists `directory: /missing-app` when that folder is
absent.

Fix: create the referenced file, update the config value, or remove the stale
reference. JSON configs cannot use `no-mistakes-disable-file`; use rule-level
`exclude` instead.
