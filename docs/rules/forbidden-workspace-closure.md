# `forbidden-workspace-closure`

Reports when a configured workspace package can reach a forbidden dependency
through its package.json dependency closure.

## Why and when

Use this rule when package manifests must prevent a selected workspace package
from bringing a prohibited package into its install/runtime closure.

## What it catches

It follows configured manifest dependency types, workspace aliases, and the
lockfile when configured, then reports a path to each forbidden package.

## Options

`packages` and `forbidden` are the selected origins and blocked package names.
`dependencyTypes` defaults to the documented manifest dependency kinds, and
optional `lockfile` adds resolved lockfile edges; `allow` records intentional
closure exceptions.

## Valid example

The compliant manifest pair below passes because the `@acme/app` closure never
reaches `@acme/secret`.

## Related rules

[`forbidden-dependencies`](forbidden-dependencies.md) enforces equivalent
source-graph boundaries; [`workspace-package-cycles`](workspace-package-cycles.md)
checks cycles in the workspace package graph.

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
