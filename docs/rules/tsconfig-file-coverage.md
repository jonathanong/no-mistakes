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

## Why and when

Use this rule when every tracked TypeScript source file must belong to a
compiler program, independent of whether CI happens to run that program.

## What it catches/requires

Every tracked `.ts`, `.mts`, `.cts`, and `.tsx` file must match at least one
tracked tsconfig `files`/`include` set or a reasoned `allow` entry.

## Options and defaults

`allow` and `auxiliaryConfigs` default to empty. `requiredBasename` defaults to
`tsconfig.dependency-cruiser.json` for auxiliary configs. Auxiliary configs may
not declare compiler `files`, `include`, `exclude`, or `references`; paths must
be repository-relative and reasons non-empty.

## Valid example

```json
{"include":["src/**/*.ts"]}
```

```text
src/index.ts
```

## Counterexample

```text
orphan.ts
```

The file is tracked beside a config that includes only `src`.

## Fix

Add the file to a compiler `include`/`files` set or add a precise `allow` entry
with why it intentionally stays outside the program.

## Suppression

Prefer a reasoned `allow` entry. Use a top-of-file file directive for a
one-off generated file; findings point to line 1.

## Related rules

[`tsconfig-gate-coverage`](tsconfig-gate-coverage.md) checks CI and local
typecheck registration; [`tsconfig-alias-folder-mapping`](tsconfig-alias-folder-mapping.md)
checks alias targets.
