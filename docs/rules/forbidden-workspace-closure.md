# `forbidden-workspace-closure`

Reports when a configured workspace package can reach a forbidden dependency
through its package.json dependency closure.

```yaml
rules:
  - rule: forbidden-workspace-closure
    scope: repository
    options:
      packages: ["@acme/app"]
      forbidden: ["@acme/secret"]
      dependencyTypes: [dependencies, devDependencies]
      lockfile: pnpm-lock.yaml
```

Counterexample:

```json
{
  "name": "@acme/app",
  "dependencies": {
    "@acme/domain": "workspace:*"
  }
}
```

```json
{
  "name": "@acme/domain",
  "dependencies": {
    "@acme/secret": "^1.0.0"
  }
}
```

Compliant example:

```json
{
  "name": "@acme/app",
  "dependencies": {
    "@acme/domain": "workspace:*"
  }
}
```

```json
{
  "name": "@acme/domain",
  "dependencies": {
    "left-pad": "^1.3.0"
  }
}
```

Fix: remove the forbidden dependency, move it outside the workspace closure,
or narrow `packages`, `forbidden`, `dependencyTypes`, or `lockfile` so the
rule only covers the packages you actually want to police.

Suppression caveat: findings point at `package.json`, so inline
`no-mistakes-disable-*` directives are not available there. Prefer a narrower
rule configuration, or suppress the enclosing file only when the manifest format
can actually carry directives.
