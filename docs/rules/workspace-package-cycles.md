# `workspace-package-cycles`

Reports dependency cycles between workspace packages.

```yaml
rules:
  - rule: workspace-package-cycles
    scope: repository
    options:
      dependencyTypes: [dependencies, devDependencies]
```

Counterexample: `@app/api` depends on `@app/domain`, while `@app/domain`
depends on `@app/api`.

Fix: extract the shared dependency, invert one dependency, or add a temporary
`allowlist` entry for an intentional cycle.
