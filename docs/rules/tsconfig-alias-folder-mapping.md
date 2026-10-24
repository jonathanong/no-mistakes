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
match the declared prefix policy. For alias `@api/<suffix>` under root `src/api`,
accepted targets are `src/api/<suffix>`, `src/api/<suffix>.<ext>`, and
`src/api/<suffix>/index.<ext>`. Wildcards such as `@api/<dir>/*` also accept
`src/api/<dir>/*.<ext>` alongside `src/api/<dir>/*`. Extensions are nonempty
filename suffixes, including compound suffixes such as `d.ts`; they cannot add
another directory or wildcard. Mismatched roots, names, and wildcard shapes
remain findings. Missing targets are findings when `checkExists` is enabled.
An existing tsconfig that cannot be loaded is a finding, because its alias
mappings were not checked. A missing tsconfig path is not.

## Options and defaults

`tsconfig` identifies the config to inspect, `baseDir` anchors relative target
folders, and `mappings` supplies `prefix`/`root` pairs. `checkExists` defaults
to `false`; enable it to require literal files to be files, directory targets
to be directories, and wildcard targets' parent directories to exist. A wildcard
with an extension does not require a matching file. These are explicit policy
options; there are no inferred alias conventions.

## Valid example

```json
// tsconfig.json
{
  "compilerOptions": {
    "paths": {
      "@api/*": ["src/api/*"],
      "@api/email": ["src/api/email/index.ts"],
      "@api/task": ["src/api/task.ts"],
      "@api/jobs/*": ["src/api/jobs/*.ts"]
    }
  }
}
```

With `baseDir: src` and `mappings: [{prefix: "@api", root: "api"}]`, the
aliases and targets agree. Findings point to the actual `paths` entry line.
For inherited paths, the file and line belong to the configuration that defines
the winning `paths` block, including the last occurrence of duplicate keys.
Rule include/exclude filters still select the configured `tsconfig` path.

## Counterexample

```json
// tsconfig.json
{"compilerOptions":{"paths":{"@api/*":["src/services/*"]}}}
```

## Fix

Rename the folder or update `compilerOptions.paths` and the mapping together;
then run the resolver check on representative imports.

## Analysis

### Effort

Runs at `max` only. Configuring `effort: low` is accepted: `check` emits an analysis warning (requested effort low; effective effort max because this rule has no low implementation) and runs the rule at `max`.

### Unanalyzable inputs

Following recall over precision, nothing is silently treated as clean ([how analysis works](../../README.md#how-analysis-works)); this rule has no unanalyzable inputs because it compares parsed tsconfig `paths` entries with the file inventory, so `options.unanalyzable` has no effect on it.

## Suppression

Adjust `mappings` for generated or package-provided aliases. Use a file
directive only for a config owned by another tool.

In a JSONC config, put `// no-mistakes-disable-next-line
tsconfig-alias-folder-mapping: reason` on the line immediately before an alias
entry to suppress only that entry. For inherited paths, put the directive in
the defining config alongside the entry.

## Related rules

[`tsconfig-file-coverage`](tsconfig-file-coverage.md) checks file membership;
[`required-entrypoint-reachability`](required-entrypoint-reachability.md)
checks runtime loading rather than alias naming.
