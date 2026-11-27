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
dependency types. A dependency is a workspace edge when its name is a
workspace package and the specifier resolves to that package. `workspace:*`
and a `workspace:` specifier that is not a relative path always name that
package. `workspace:` followed by a path starting with `.` counts only when
that path is the package directory. `file:`, `link:`, and `portal:` count
only for a relative path that starts with `.` (not an archive) when that path,
resolved from the depending package, is the package directory. A satisfying
version range or `*` is a workspace edge even when pnpm `linkWorkspacePackages`
is off; this rule does not read that setting. A dependency is skipped when it
is proven to resolve outside the workspace: `catalog:`, an `npm:` alias,
`jsr:`, `github:`, `gist:`, `bitbucket:`, `gitlab:`, a git or URL specifier, a
path that is not the workspace package directory, or a range the workspace
version does not satisfy (including a package that has no `version`). Findings
report the package-level cycle path.

## Options and defaults

`dependencyTypes` selects dependency fields. When omitted or empty it is
`dependencies`, `devDependencies`, `peerDependencies`, and
`optionalDependencies`. `allowlist` defaults to empty and may hold documented
intentional cycles.

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

Use a narrow, reasoned `allowlist` entry for a temporary cycle, including in
generated manifests. The cycle check builds its graph from valid
`package.json` files. Adding a comment makes a manifest invalid JSON and
removes the package from cycle analysis.

## Related rules

[`production-dependency-declarations`](production-dependency-declarations.md)
checks runtime fields; [`strict-package-layout`](strict-package-layout.md)
keeps package boundaries explicit.

Unused explicit exception entries produce non-failing `unused-allow-entry` advisories by default when their complete selector scope was checked. Remove stale entries; narrowed or unproven scopes stay silent. An entry is used only when it suppresses a finding, and overlapping exceptions are all credited.

Repository `reportUnusedAllowEntries: report` makes these diagnostics fail;
`ignore` omits them. See [allow-entry hygiene](../configuration/suppressions.md#allow-entry-hygiene).
