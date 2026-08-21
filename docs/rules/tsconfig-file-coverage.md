# `tsconfig-file-coverage`

Requires every tracked TypeScript file (`.ts`, `.mts`, `.cts`, `.tsx`) to
belong to at least one tracked tsconfig `files`/`include` set, or to a reasoned
`allow` entry. This is the file-universe check: it does not inspect CI or local
`tsc` registration. Use [`tsconfig-gate-coverage`](tsconfig-gate-coverage.md)
for that.

The filesystem dispatcher supplies the Git-tracked inventory when a discovery
snapshot is available, so untracked scratch files are ignored. Direct callers
without a snapshot should pass tracked paths themselves.

```yaml
rules:
  - rule: tsconfig-file-coverage
    scope: repository
    options:
      allow:
        - path: scripts/generate.ts
          reason: generated entrypoint kept outside the app program
      auxiliaryConfigs:
        - path: tsconfig.dependency-cruiser.json
          reason: dependency-cruiser resolver config
      requiredBasename: tsconfig.dependency-cruiser.json
```

Tracked `tsconfig.json` / `tsconfig.*.json` files (not under `node_modules`)
are the candidate programs, including tsconfigs that a rule `include` filter
would otherwise drop. Repositories with no tracked tsconfig produce no
source-coverage findings, so fixtures without compiler config stay silent.
Configured `allow` and `auxiliaryConfigs` entries are still validated.
`auxiliaryConfigs` entries are not compiler programs: they must not declare
`files`, `include`, `exclude`, or `references`, and their basename defaults to
`tsconfig.dependency-cruiser.json`. Empty `reason` values, stale paths, and
absolute or `..` option paths are findings.

v1 membership uses `TsConfigCatalog::project_source_membership` for
include/files/exclude matching. It does not close over the static import graph,
so a file imported only by a program member is still uncovered unless that file
itself matches a tsconfig `files`/`include` set.

Counterexample: `orphan.ts` sits next to a program that only includes `src`.

```json
{
  "include": ["src"]
}
```

```ts
export const orphan = 1;
```

Fix: add the file to a tsconfig `include`/`files` set, or give it a reasoned
`allow` entry.

```yaml
allow:
  - path: orphan.ts
    reason: generated script excluded from the app program
```

Findings use line 1 of the uncovered file or option path. Use a top-of-file
`no-mistakes-disable-file tsconfig-file-coverage` directive, or
`no-mistakes-disable-line` on line 1, for a one-off exception.
