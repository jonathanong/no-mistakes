# `strict-package-layout`

Enforces configured package directory layout.

```yaml
rules:
  - rule: strict-package-layout
    scope: repository
    options:
      roots: [packages]
      requiredFiles: [package.json, README.md]
```

Counterexample: a package missing required files or containing banned paths.

Fix: add/move files to the configured package layout.
