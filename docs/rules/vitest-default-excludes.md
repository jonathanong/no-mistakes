# `vitest-default-excludes`

## Why and when

Declaring `test.exclude` replaces Vitest's default test-discovery exclusions.
An array that drops those defaults can cause Vitest to scan dependency trees or
Git metadata. Enable this rule to catch that change in a selected config without
running Vitest.

## Configuration

```yaml
version: 2
rules:
  - rule: vitest-default-excludes
    scope: repository
    options:
      config: vitest.config.ts
```

`config` is a required path relative to the repository root. The optional
`requiredGlobs` array replaces the required list, which defaults to
`**/node_modules/**` and `**/.git/**`. Each entry must be a nonempty string.
The rule compares strings exactly and deduplicates repeated entries. An empty
array disables the check. Extra exclusions are allowed. Shared rule severity,
message and file filters apply.

## What it catches

Use an ECMAScript module with a literal object as its default export, either
directly or as the argument to `defineConfig` or `defineProject`. The wrapper
must be a named value import from `vitest/config`. Renamed imports are
supported. Inline entries in `test.projects` can use the same literal object
and wrapper forms.

The rule checks explicit `test.exclude` arrays in those objects. Leaving
`exclude` unset preserves Vitest's defaults. The recognized
`configDefaults.exclude` spread satisfies all configured
`requiredGlobs`, including custom entries; no additional strings are needed.
Recognition is syntactic and required glob matching is exact-string: a
handwritten string is checked as written. For example, Vitest's combined
default pattern `**/.{idea,git,cache,output,temp}/**` does not satisfy the
separate required string `**/.git/**`.

The rule skips callback configs, CommonJS exports, re-exported or indirect root
bindings, external project config files, coverage exclusions, and other
properties named `exclude`. Arrays with unknown entries or unrecognized
spreads, as well as conditional or call-based `exclude` values, are also
skipped. The rule does not merge config roots through `mergeConfig`, recognize
`defineConfig` imported from `vite`, or inspect Vitest workspace files.

## Counterexample

```ts
// vitest.config.ts
import { defineConfig } from 'vitest/config';

export default defineConfig({
  test: { exclude: ['**/generated/**'] },
});
```

The finding names the missing exclusions and points to the `exclude` property.

## Fix

Add the missing strings explicitly or spread `configDefaults.exclude` imported
from `vitest/config`. A renamed import works as well. The recognized spread
satisfies all configured `requiredGlobs`, including custom entries.

## Valid example

```ts
// vitest.config.ts
import { configDefaults as defaults, defineConfig } from 'vitest/config';

export default defineConfig({
  test: { exclude: [...defaults.exclude] },
});
```

## Suppression

Use `// no-mistakes-disable-file vitest-default-excludes` for the config file.
Use `// no-mistakes-disable-line vitest-default-excludes` on the reported line or
`// no-mistakes-disable-next-line vitest-default-excludes` directly above it.

## Rust API

Call `codebase::rules::vitest_default_excludes::check_with_files(root, &config, &files)`
with a parsed no-mistakes configuration and a file inventory. It prepares the
selected source facts for that invocation, applies source suppressions, and
returns the findings.

## Node API

The same configured rule runs through async `check()` and
`analyzeProject()` check reports. `VitestDefaultExcludesOptions` is exported
from `no-mistakes` for rule configuration.

## Related rules

Use the broader Vitest project policy to enforce literal project settings or
compare settings across projects. [`vitest-project-policy`](vitest-project-policy.md)

Use project mapping to check that each Vitest test belongs to exactly one
configured project. [`vitest-project-mapping`](vitest-project-mapping.md)
