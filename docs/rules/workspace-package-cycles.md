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

## Why and when

Use this rule when workspace packages should form an acyclic dependency layer so
they can be built, published, and tested independently.

## What it catches/requires

The workspace package graph must contain no cycles among the configured
dependency types. A dependency is a workspace edge only when its name is a
workspace package and its specifier uses the `workspace:` protocol. A registry
range, version catalog (`catalog:`), `file:` path, or other external specifier
stays outside the graph, including when that name matches a workspace package.
Findings report the package-level cycle path.

## Options and defaults

`dependencyTypes` selects dependency fields and defaults to the repository's
configured dependency policy; `allowlist` defaults to empty and may hold
documented intentional cycles.

## Valid example

`@app/api` depends on `@app/domain`, which depends on `@app/types`. The
workspace graph is acyclic.

```json
// package.json
{ "name": "root", "private": true, "workspaces": ["packages/*"] }

// packages/api/package.json
{ "name": "@app/api", "dependencies": { "@app/domain": "workspace:*" } }

// packages/domain/package.json
{ "name": "@app/domain", "dependencies": { "@app/types": "workspace:*" } }

// packages/types/package.json
{ "name": "@app/types" }
```

## Counterexample

`@app/api` depends on `@app/domain`, while `@app/domain` depends on
`@app/api`.

```json
// package.json
{ "name": "root", "private": true, "workspaces": ["packages/*"] }

// packages/api/package.json
{ "name": "@app/api", "dependencies": { "@app/domain": "workspace:*" } }

// packages/domain/package.json
{ "name": "@app/domain", "dependencies": { "@app/api": "workspace:*" } }
```

## Fix

Extract shared types or utilities, invert one dependency through an interface,
or split the package boundary so the runtime graph becomes directional.

## Analysis

### Effort

Runs at `max` only. Configuring `effort: low` is accepted: `check` emits an analysis warning (requested effort low; effective effort max because this rule has no low implementation) and runs the rule at `max`.

### Unanalyzable inputs

Following recall over precision, nothing is silently treated as clean ([how analysis works](../../README.md#how-analysis-works)); this rule has no unanalyzable inputs because it detects cycles in the workspace package graph built from parsed manifests, so `options.unanalyzable` has no effect on it.

## Suppression

Prefer a narrow, reasoned `allowlist` entry for a temporary cycle. Use a file
directive only for generated manifests.

## Related rules

[`production-dependency-declarations`](production-dependency-declarations.md)
checks runtime fields; [`strict-package-layout`](strict-package-layout.md)
keeps package boundaries explicit.
