# `package-json-nested-workspace-coverage`

Requires each configured package root to list exactly the explicit workspace
directories for matching dependencies declared by that root and its descendant
package manifests.

```yaml
rules:
  - rule: package-json-nested-workspace-coverage
    scope: repository
    options:
      roots: [apps, lambdas/*]
      dependencyNamePrefixes: ['@shared/']
      dependencyFields: [dependencies, devDependencies, optionalDependencies]
```

`roots` accepts literal directories and directory globs. Each matching tracked
`package.json` is a separate root. `dependencyFields` is optional and defaults
to `dependencies`, `devDependencies`, and `optionalDependencies`.

`no-mistakes-config` validates every `roots` entry against the tracked
repository inventory, so a deleted directory or a glob that matches nothing
fails before it can silently disable nested-workspace coverage.

Counterexample: `apps/api/package.json` declares `@shared/utils`, but
`apps/package.json` omits `../packages/utils` from `workspaces`. A wildcard
such as `../packages/*` that covers a matching dependency package is also
rejected: list the directory explicitly. If a matching dependency has no
unique visible `package.json` target, the rule fails closed.

Fix: add the exact POSIX-relative package directory to the root's `workspaces`
array, remove a stale matching entry, replace a wildcard with explicit paths,
or make the dependency target visible. Standard `no-mistakes-disable-file`,
`no-mistakes-disable-line`, and `no-mistakes-disable-next-line` directives
apply to the root manifest's `workspaces` field.
