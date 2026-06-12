# `config-path-references`

Validates path strings stored inside structured YAML or JSON config files.

```yaml
rules:
  - rule: config-path-references
    scope: repository
    options:
      files: [config/app.yml]
      keys: [paths.requiredFiles, paths.testGlobs]
      baseDir: config-file
      allowGlobs: true
```

Counterexample: `config/app.yml` contains `paths.requiredFiles:
["missing.json"]`, and `config/missing.json` does not exist.

Fix: create the referenced file, update the config value, or remove the stale
reference.
