# `package-json-workspace-coverage`

## Why and when

Use this rule when every package directory must be declared in the root
workspace list so installs and graph analysis agree.

## What it catches

It reports selected package directories omitted from configured workspace globs
or workspace entries that do not correspond to a package directory. `*` does
not match across `/`, so `packages/*` covers `packages/api` and leaves
`packages/group/app` uncovered. Each non-negated pattern is checked on its
own: a pattern that matches a package directory is not "unmatched" when a
later `!` exclusion removes that directory from the workspace. Negated
patterns are not reported. The finding is attached to `pnpm-workspace.yaml`
when that file is in the file list, and otherwise to the root `package.json`.

A root `pnpm-workspace.yaml` may `include` a local `./` or `../` YAML file
that owns `packages`. Patterns in that file are relative to the included
file, then normalized, and a glob that escapes the workspace root is dropped.
`packages` delegated to a local file does not fall back to `["*"]` when the
file omits the key. The same setting in the root file and a local include is
a conflict: the check reports that `packages` is claimed twice and does not
guess which list wins. Coverage is still judged against the root file's own
`packages` list, so a directory that list omits stays a finding. An omitted
`packages` key with no local include still means `["*"]`.

## Options

`packageRoots` selects package directories, `allowlist` names intentional
non-workspace package manifests, and `requireNamedPackage` defaults to `false`.
The rule reads the repository workspace configuration; it has no
`workspaceFile` or `workspaceField` rule options. Shared rule `include`/`exclude`
filters still apply.

## Valid example

A package beneath `packages/*` with a matching root workspace glob passes.

```json
// package.json
{ "name": "root", "private": true, "workspaces": ["packages/*"] }

// packages/web/package.json
{ "name": "web", "version": "1.0.0" }
```

## Related rules

[`package-json-nested-workspace-coverage`](package-json-nested-workspace-coverage.md)
checks nested workspace declarations; [`strict-package-layout`](strict-package-layout.md)
checks each package's internal layout.

## Counterexample

```json
// package.json
{ "name": "root", "private": true, "workspaces": ["packages/missing"] }

// packages/web/package.json
{ "name": "web", "version": "1.0.0" }
```

## Analysis

### Effort

Runs at `max` only. Configuring `effort: low` is accepted: `check` emits an analysis warning (requested effort low; effective effort max because this rule has no low implementation) and runs the rule at `max`.

### Unanalyzable inputs

Following recall over precision, nothing is silently treated as clean ([how analysis works](../../README.md#how-analysis-works)); this rule has no unanalyzable inputs because it compares parsed package manifests with workspace globs, so `options.unanalyzable` has no effect on it.

## Suppression

Workspace manifest findings are file-level. Prefer narrowing `packageRoots` or
the workspace selection for a deliberate exceptional package.

Reports package directories under configured roots that are not covered by the
repository workspace config.

```yaml
rules:
  - rule: package-json-workspace-coverage
    scope: repository
    options:
      packageRoots: [packages, apps]
      requireNamedPackage: true
```

`requireNamedPackage: true` reports manifests that omit `name` or set an empty
`"name": ""` instead of skipping them.

Counterexample: `packages/api/package.json` exists, but the root `workspaces`
or `pnpm-workspace.yaml` patterns do not include `packages/api`. An unnamed
`package.json` (`{}` or `"name": ""`) is also a finding when
`requireNamedPackage` is true.

Fix: add the package directory to the workspace patterns, move the package
outside the configured package roots, or add a deliberate `allowlist` entry.
For unnamed manifests, set a non-empty `name`.

## Limitations

Settings owned by a pnpm config dependency cannot be loaded without an install.
This rule does not read `node_modules`. A source whose key does not start with
`./` or `../` is ignored. When `packages` is delegated only to that kind of
source, the omitted-packages default `["*"]` stays in effect.

```yaml
include:
  "@acme/pnpm-policy":
    - packages
```
