# `tsconfig-alias-folder-mapping`

Enforces consistency between TypeScript path aliases and target folders.

```yaml
rules:
  - rule: tsconfig-alias-folder-mapping
    scope: repository
    options:
      tsconfig: tsconfig.json
      baseDir: src
      mappings:
        - prefix: "@api"
          root: api
```

Counterexample: `@api/*` pointing to a folder that does not exist or does not
match the alias prefix policy.

Fix: update `compilerOptions.paths` or rename folders to match.

## Why and when

Use this rule when aliases are part of the repository's import contract and
must continue to point at real, intentionally named folders.

## What it catches/requires

Each configured alias prefix must map to the configured target root/folder and
match the declared prefix policy. Missing targets and mismatched folder names
are findings.

## Options and defaults

`tsconfig` identifies the config to inspect, `baseDir` anchors relative target
folders, and `mappings` supplies `prefix`/`root` pairs. `checkExists` defaults
to `false`; enable it to require each expected target directory on disk. These
are explicit policy options; there are no inferred alias conventions.

## Valid example

```json
{"compilerOptions":{"paths":{"@api/*":["src/api/*"]}}}
```

With `baseDir: src` and `mappings: [{prefix: "@api", root: "api"}]`, the
alias and folder agree.

## Counterexample

```json
{"compilerOptions":{"paths":{"@api/*":["src/services/*"]}}}
```

## Fix

Rename the folder or update `compilerOptions.paths` and the mapping together;
then run the resolver check on representative imports.

## Suppression

Adjust `mappings` for generated or package-provided aliases. Use a file
directive only for a config owned by another tool.

## Related rules

[`tsconfig-file-coverage`](tsconfig-file-coverage.md) checks file membership;
[`required-entrypoint-reachability`](required-entrypoint-reachability.md)
checks runtime loading rather than alias naming.
