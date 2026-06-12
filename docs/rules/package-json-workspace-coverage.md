# `package-json-workspace-coverage`

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

Counterexample: `packages/api/package.json` exists, but the root `workspaces`
or `pnpm-workspace.yaml` patterns do not include `packages/api`.

Fix: add the package directory to the workspace patterns, move the package
outside the configured package roots, or add a deliberate `allowlist` entry.
