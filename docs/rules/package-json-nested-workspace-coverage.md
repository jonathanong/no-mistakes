# `package-json-nested-workspace-coverage`

## Why and when

Use this rule when nested workspace configuration must exactly represent the
packages a selected workspace depends on.

## What it requires

It reports missing, extra, or mis-scoped nested workspace entries relative to
the configured dependency package inventory.

## Options

`roots` selects package manifests, `dependencyNamePrefixes` limits dependency
names, and `dependencyFields` selects manifest fields. `dependencyFields`
defaults to `dependencies`, `devDependencies`, and `optionalDependencies`;
the other lists default to empty. Shared rule `include`/`exclude` filters still
apply.

```yaml
rules:
  - rule: package-json-nested-workspace-coverage
    scope: repository
    options:
      roots: [apps]
      dependencyNamePrefixes: ['@shared/']
```

`roots` accepts literal directories and directory globs. Each matching tracked
`package.json` is a separate root.

`no-mistakes-config` validates every `roots` entry against the tracked
repository inventory, so a deleted directory or a glob that matches nothing
fails before it can silently disable nested-workspace coverage.

The following block is an extended reference of glob roots and field
selection. The Options example above is the one Valid and Counterexample
execute.

```yaml
rules:
  - rule: package-json-nested-workspace-coverage
    scope: repository
    options:
      roots: [apps, lambdas/*]
      dependencyNamePrefixes: ['@shared/']
      dependencyFields: [dependencies, devDependencies, optionalDependencies]
```

## Valid example

A selected package whose nested workspace entries exactly cover its configured
workspace dependencies passes.

```json
// apps/package.json
{ "name": "app", "private": true, "workspaces": ["../packages/utils"] }

// apps/api/package.json
{ "name": "api", "dependencies": { "@shared/utils": "workspace:*" } }

// packages/utils/package.json
{ "name": "@shared/utils", "version": "1.0.0" }
```

## Counterexample

`apps/api/package.json` declares `@shared/utils`, but `apps/package.json` omits
`../packages/utils` from `workspaces`. A wildcard such as `../packages/*` that
covers a matching dependency package is also rejected: list the directory
explicitly. If a matching dependency has no unique visible `package.json`
target, the rule fails closed.

```json
// apps/package.json
{ "name": "app", "private": true, "workspaces": [] }

// apps/api/package.json
{ "name": "api", "dependencies": { "@shared/utils": "workspace:*" } }

// packages/utils/package.json
{ "name": "@shared/utils", "version": "1.0.0" }
```

## Related rules

[`package-json-workspace-coverage`](package-json-workspace-coverage.md) checks
top-level workspace membership; [`workspace-package-cycles`](workspace-package-cycles.md)
checks the resulting package graph.

Requires each configured package root to list exactly the explicit workspace
directories for matching dependencies declared by that root and its descendant
package manifests.

Fix: add the exact POSIX-relative package directory to the root's `workspaces`
array, remove a stale matching entry, replace a wildcard with explicit paths,
or make the dependency target visible. Standard `no-mistakes-disable-file`,
`no-mistakes-disable-line`, and `no-mistakes-disable-next-line` directives
apply to the root manifest's `workspaces` field.

## Analysis

### Effort

Runs at `max` only. Configuring `effort: low` is accepted: `check` emits an analysis warning (requested effort low; effective effort max because this rule has no low implementation) and runs the rule at `max`.

### Unanalyzable inputs

Following recall over precision, nothing is silently treated as clean ([how analysis works](../../README.md#how-analysis-works)); this rule has no unanalyzable inputs because it compares parsed package manifests with workspace globs, so `options.unanalyzable` has no effect on it.

## Suppression

Manifest findings are file-level. Narrow `roots`, dependency fields, or package
selection for an intentional exception instead of suppressing generated JSON.
