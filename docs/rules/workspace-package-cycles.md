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
dependency types. Findings report the package-level cycle path.

## Options and defaults

`dependencyTypes` selects dependency fields and defaults to the repository's
configured dependency policy; `allowlist` defaults to empty and may hold
documented intentional cycles.

## Valid example

```text
@app/api -> @app/domain -> @app/types
```

## Counterexample

```text
@app/api -> @app/domain -> @app/api
```

## Fix

Extract shared types or utilities, invert one dependency through an interface,
or split the package boundary so the runtime graph becomes directional.

## Suppression

Prefer a narrow, reasoned `allowlist` entry for a temporary cycle. Use a file
directive only for generated manifests.

## Related rules

[`production-dependency-declarations`](production-dependency-declarations.md)
checks runtime fields; [`strict-package-layout`](strict-package-layout.md)
keeps package boundaries explicit.
