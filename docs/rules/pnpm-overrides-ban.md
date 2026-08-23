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
