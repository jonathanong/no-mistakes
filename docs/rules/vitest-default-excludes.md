# `vitest-default-excludes`

## Why and when

Declaring `test.exclude` replaces Vitest's default test-discovery exclusions.
An array that drops those defaults can cause Vitest to scan dependency trees or
Git metadata. Enable this rule to catch that change in a selected config or
shared config factory modules without running Vitest.

## Configuration

```yaml
version: 2
rules:
  - rule: vitest-default-excludes
    scope: repository
    options:
      config: vitest.config.ts
      files:
        - "config/vitest-*.ts"
```

At least one of `config` or `files` is required:

- `config`: an explicit Vitest config path relative to the repository root.
- `files`: a list of repository-relative module globs selecting TypeScript or
  JavaScript modules from the prepared file inventory (such as shared Vitest
  config factories). Every entry must be a nonempty glob. Modules outside
  `files` are not checked unless selected by `config`.
- `requiredGlobs`: optional array that replaces the required list, which
  defaults to `**/node_modules/**` and `**/.git/**`. Each entry must be a
  nonempty string. The rule compares strings exactly and deduplicates repeated
  entries. An empty array disables the check. Extra exclusions are allowed.

Shared rule severity, message, and `include` / `exclude` file filters apply to
both `config` and `files`.

## What it catches

In the file named by `config`, use an ECMAScript module with a literal object
as its default export, either directly or as the argument to `defineConfig` or
`defineProject`. The wrapper must be a named value import from `vitest/config`.
Renamed imports are supported. Inline entries in `test.projects` can use the
same literal object and wrapper forms.

In modules matched by `files`, the rule checks every `test`-shaped object
literal (an object literal with a `test` property whose value is an object
literal holding `exclude`), including objects returned by config factory
functions or passed to wrappers. If a file is both named by `config` and
matched by `files`, its `test`-shaped object literals are checked once without
duplicate findings.

In both modes, the rule checks explicit `test.exclude` arrays. Leaving
`exclude` unset preserves Vitest's defaults. A spread can use a literal string
array, a local `const` initialized to one, or a named import of an
`export const` literal array from a relative module in the prepared file
inventory. Renamed imports and `as const` array initializers work too. The
rule follows lexical bindings and rejects arrays that are mutated or passed
around in the declaring or importing module. The recognized
`configDefaults.exclude` spread satisfies all configured `requiredGlobs`,
including custom entries; no additional strings are needed. Recognition is
syntactic and required glob matching is exact-string: a handwritten string is
checked as written. For example, Vitest's combined default pattern
`**/.{idea,git,cache,output,temp}/**` does not satisfy the separate required
string `**/.git/**`.

In `config` mode, the rule skips callback configs, CommonJS exports,
re-exported or indirect root bindings, external project config files,
`mergeConfig`, `defineConfig` imported from `vite`, and Vitest workspace files.
In both modes, it skips coverage exclusions, unrelated properties named
`exclude`, properties followed by a later spread or dynamic key, arrays with unknown
entries and no spread, and conditional or call-based `exclude` values. An
array with a spread that cannot be resolved reports an `unanalyzable` finding
on the `exclude` property. Rewrite the spread as a literal string array or
reference a statically known const array. Package imports, re-exports, and
runtime expressions in spreads are unanalyzable.

## Counterexample

Default-exported config:

```ts
// vitest.config.ts
import { defineConfig } from 'vitest/config';

const extra = ['**/generated/**'];
export default defineConfig({
  test: { exclude: ['**/node_modules/**', ...extra] },
});
```

Shared config factory matched by `options.files`:

```ts
// config/vitest-factory.ts
export function createVitestConfig() {
  return {
    test: { exclude: ['custom/**'] },
  };
}
```

Each finding names the missing exclusions and points to the `exclude` property.

## Fix

Add the missing strings explicitly or spread `configDefaults.exclude` imported
from `vitest/config`. A renamed import works as well. The recognized spread
satisfies all configured `requiredGlobs`, including custom entries.

## Valid example

Default-exported config:

```ts
// vitest.config.ts
import { configDefaults as defaults, defineConfig } from 'vitest/config';

export default defineConfig({
  test: { exclude: [...defaults.exclude] },
});
```

Shared config factory matched by `options.files`:

```ts
// config/vitest-factory.ts
import { configDefaults } from 'vitest/config';

export function createVitestConfig() {
  return {
    test: { exclude: [...configDefaults.exclude, 'custom/**'] },
  };
}
```

## Suppression

Use `// no-mistakes-disable-file vitest-default-excludes` for a config or
factory module. Use `// no-mistakes-disable-line vitest-default-excludes` on
the reported line or `// no-mistakes-disable-next-line vitest-default-excludes`
directly above it:

```ts
// config/vitest-factory.ts
export function createScopedConfig() {
  return {
    test: {
      // no-mistakes-disable-next-line vitest-default-excludes
      exclude: ['custom/**'],
    },
  };
}
```

## Rust API

Call `codebase::rules::vitest_default_excludes::check_with_files(root, &config, &files)`
with a parsed no-mistakes configuration and a file inventory. It prepares the
selected source facts for that invocation, applies source suppressions, and
returns the findings.

## Node API

The same configured rule runs through async `check()` and
`analyzeProject()` check reports. `VitestDefaultExcludesOptions` is exported
from `no-mistakes` for rule configuration (`config`, `files`, and
`requiredGlobs`).

## Related rules

Use the broader Vitest project policy to enforce literal project settings or
compare settings across projects. [`vitest-project-policy`](vitest-project-policy.md)

Use project mapping to check that each Vitest test belongs to exactly one
configured project. [`vitest-project-mapping`](vitest-project-mapping.md)

## Invalid options

An invalid option such as an empty `files` glob is a configuration error:
`check` validates options before any analysis runs, exits with status 2 and
empty stdout, and prints semantic option diagnostics sorted to stderr (naming
the config file, rule, option path, rejected value, and constraint). No partial
report or `incomplete` entry is emitted, and the async Node API rejects with the
same message.
