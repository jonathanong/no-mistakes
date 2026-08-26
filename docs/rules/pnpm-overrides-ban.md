# `pnpm-overrides-ban`

Ban pnpm dependency version overrides. `packageExtensions` remains allowed
because it corrects dependency metadata without forcing versions.

```yaml
rules:
  - rule: pnpm-overrides-ban
    scope: repository
```

Counterexample: `pnpm-workspace.yaml` with a top-level `overrides` map, or a
workspace `package.json` with top-level `overrides` or `pnpm.overrides`.

Fix: remove the override and fix dependency metadata upstream, or upgrade the
parent dependency instead. Keep `packageExtensions` when a package needs
corrected peer metadata.

## Why and when

Use this rule in workspaces where reproducible dependency resolution matters
more than forcing a transitive version from the root manifest.

## What it catches/requires

It rejects pnpm `overrides` in `pnpm-workspace.yaml` and workspace manifests,
including `pnpm.overrides`. `packageExtensions` remains allowed because it
describes missing metadata rather than selecting a version.

## Options and defaults

There are no rule-local options. The rule scans the configured repository
workspace manifests.

## Valid example

```yaml
packageExtensions:
  "react-dom@*":
    peerDependencies:
      react: "*"
```

## Counterexample

```yaml
overrides:
  lodash: 4.17.21
```

## Fix

Resolve the version conflict in the owning package, upgrade the dependency, or
correct its metadata with `packageExtensions`.

## Suppression

Use `no-mistakes-disable-file pnpm-overrides-ban` only when an external release
constraint makes the override unavoidable; document that constraint nearby.

## Related rules

[`version-pin-consistency`](version-pin-consistency.md) keeps intentional pins
in sync, while [`production-dependency-declarations`](production-dependency-declarations.md)
checks that runtime imports are declared in the right field.
