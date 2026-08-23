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
        - pnpm-workspace-filters
        - no-mistakes
```

| Preset                   | Files                                                                       | Required paths                                                                        |
| ------------------------ | --------------------------------------------------------------------------- | ------------------------------------------------------------------------------------- |
| `oxlintrc`               | `.oxlintrc.json` / `.oxlintrc.jsonc` (including nested)                     | relative `jsPlugins[].specifier`, non-glob `overrides.files`, `rules.*.baseline[][0]` |
| `knip`                   | `knip.json` / `knip.jsonc`                                                  | `workspaces.{name}.entry` and `.project`, prefixed by the workspace path unless `.`   |
| `dependabot`             | `.github/dependabot.yml`                                                    | `updates[].directory`                                                                 |
| `sgconfig`               | `sgconfig.yml`                                                              | `ruleDirs`                                                                            |
| `syncpack`               | `.syncpackrc.json`                                                          | `source` globs                                                                        |
| `coverage-rules`         | `.coverage-rules.yml`                                                       | `rules[].paths`                                                                       |
| `pnpm-workspace-filters` | `.github/workflows/*.{yml,yaml}` and `.github/actions/**/action.{yml,yaml}` | `pnpm --filter ./path...` selectors in shell commands                                 |
| `no-mistakes`            | root `.no-mistakes.yml` / `.no-mistakes.yaml`                              | schema-known project, test config, TestPlan trigger, and rule-option paths             |

Counterexample: `config/app.yml` contains `paths.requiredFiles:
["missing.json"]`, and `config/missing.json` does not exist. Or
`.github/dependabot.yml` lists `directory: /missing-app` when that folder is
absent.

Fix: create the referenced file, update the config value, or remove the stale
reference. JSON configs cannot use `no-mistakes-disable-file`; use rule-level
`exclude` instead.

The `pnpm-workspace-filters` preset recognizes quoted and braced path selectors,
ellipsis selectors, wildcard filters, and selectors split across YAML block-scalar
commands. A selector guarded by `-f`, `-d`, or `test` is treated as optional and
is ignored when its path is not present.

The `no-mistakes` preset validates repository paths that the no-mistakes schema
defines as required. It includes project roots, test runner configs, Playwright
roots and helpers, TestPlan project and named-trigger paths, and path-valued rule
options such as `tsconfig`, `roots`, and package roots. Negative selectors,
optional globs, and exclusion-only paths are ignored.
