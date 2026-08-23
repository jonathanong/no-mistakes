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

`requireNamedPackage: true` reports manifests that omit `name` or set an empty
`"name": ""` instead of skipping them.

Counterexample: `packages/api/package.json` exists, but the root `workspaces`
or `pnpm-workspace.yaml` patterns do not include `packages/api`. An unnamed
`package.json` (`{}` or `"name": ""`) is also a finding when
`requireNamedPackage` is true.

Fix: add the package directory to the workspace patterns, move the package
outside the configured package roots, or add a deliberate `allowlist` entry.
For unnamed manifests, set a non-empty `name`.
