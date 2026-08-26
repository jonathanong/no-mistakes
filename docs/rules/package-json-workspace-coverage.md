# `package-json-workspace-coverage`

## Why and when

Use this rule when every package directory must be declared in the root
workspace list so installs and graph analysis agree.

## What it catches

It reports selected package directories omitted from configured workspace globs
or workspace entries that do not correspond to a package directory.

## Options

`packageRoots`, `workspaceFile`, `workspaceField`, `include`, and `exclude`
select the package inventory and manifest representation; defaults are the
documented root `package.json` workspace conventions.

## Valid example

A package beneath `packages/*` with a matching root workspace glob passes.

## Related rules

[`package-json-nested-workspace-coverage`](package-json-nested-workspace-coverage.md)
checks nested workspace declarations; [`strict-package-layout`](strict-package-layout.md)
checks each package's internal layout.

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
