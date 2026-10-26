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
      dependencyTypes: [dependencies]
```

## Why and when

Use this rule when package manifests must prevent a selected workspace package
from bringing a prohibited package into its install/runtime closure.

## What it catches

It follows configured manifest dependency types, workspace aliases, and the
lockfile when configured, then reports a path to each forbidden package.
`catalog:` and `catalog:<name>` follow the pnpm catalog when that entry is
`workspace:*`, `workspace:~`, `workspace:^`, or a relative `workspace:`,
`file:`, or `link:` path. The catalog is read from the root
`pnpm-workspace.yaml` and from a local `include` that owns `catalog` or
`catalogs`. A relative catalog path is resolved from the file that defines
it. `catalog` and `catalogs.default` are the same default catalog. A setting
claimed by both the root file and a local include is not walked.
pnpm lockfile edges come from the project document. The env document's `.`
importer, used for config and package-manager dependencies, is not a workspace root.
A lockfile that cannot be parsed, or whose leading document is not an env
lockfile, is reported instead of "no pnpm importers".

## Options

`packages` and `forbidden` are the selected origins and blocked package names.
`dependencyTypes` defaults to the documented manifest dependency kinds, and
optional `lockfile` adds resolved lockfile edges; `allow` records intentional
closure exceptions.

## Valid example

The compliant manifest pair below passes because the `@acme/app` closure never
reaches `@acme/secret`.

```json
// packages/app/package.json
{ "name": "@acme/app", "version": "1.0.0", "dependencies": { "@acme/ui": "workspace:*" } }

// packages/ui/package.json
{ "name": "@acme/ui", "version": "1.0.0" }
```


## Counterexample

```json
// packages/app/package.json
{ "name": "@acme/app", "version": "1.0.0", "dependencies": { "@acme/domain": "workspace:*" } }

// packages/domain/package.json
{ "name": "@acme/domain", "version": "1.0.0", "dependencies": { "@acme/secret": "^1.0.0" } }
```

## Analysis

### Effort

Runs at `max` only. Configuring `effort: low` is accepted: `check` emits an analysis warning (requested effort low; effective effort max because this rule has no low implementation) and runs the rule at `max`.

### Unanalyzable inputs

Following recall over precision, nothing is silently treated as clean ([how analysis works](../../README.md#how-analysis-works)); this rule has no unanalyzable inputs because it judges workspace dependency closure from parsed package manifests, so `options.unanalyzable` has no effect on it.

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

## Limitations

A catalog entry that is only a semver range is not walked, because
`linkWorkspacePackages` is not read. `workspace:^1.2.3` is a range, not one of
the three workspace wildcards.

```yaml
catalog:
  "@acme/domain": ^1.0.0
```

```json
{
  "dependencies": {
    "@acme/domain": "catalog:"
  }
}
```

Settings owned by a pnpm config dependency cannot be loaded without an install.
This rule does not read `node_modules`. A source whose key does not start with
`./` or `../` is ignored.

```yaml
include:
  "@acme/pnpm-policy":
    - catalog
```
