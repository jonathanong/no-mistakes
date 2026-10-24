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
are findings. An existing tsconfig that cannot be loaded is a finding, because
its alias mappings were not checked. A missing tsconfig path is not.

## Options and defaults

`tsconfig` identifies the config to inspect, `baseDir` anchors relative target
folders, and `mappings` supplies `prefix`/`root` pairs. `checkExists` defaults
to `false`; enable it to require each expected target directory on disk. These
are explicit policy options; there are no inferred alias conventions.

## Valid example

```json
// tsconfig.json
{"compilerOptions":{"paths":{"@api/*":["src/api/*"]}}}
```

With `baseDir: src` and `mappings: [{prefix: "@api", root: "api"}]`, the
alias and folder agree.

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

Following recall over precision, nothing is silently treated as clean ([how analysis works](README.md#how-analysis-works)); this rule has no unanalyzable inputs because it compares parsed tsconfig `paths` entries with the file inventory, so `options.unanalyzable` has no effect on it.

## Suppression

Adjust `mappings` for generated or package-provided aliases. Use a file
directive only for a config owned by another tool.

## Related rules

[`tsconfig-file-coverage`](tsconfig-file-coverage.md) checks file membership;
[`required-entrypoint-reachability`](required-entrypoint-reachability.md)
checks runtime loading rather than alias naming.
