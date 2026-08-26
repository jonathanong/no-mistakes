# `required-companion-imports`

Requires selected source files to have at least one companion file matching a
configured glob, and requires that companion file to import the expected source
specifier.

```yaml
rules:
  - rule: required-companion-imports
    scope: repository
    options:
      sourceDirs: [src/components]
      sourceGlobs: ["src/components/**/*.tsx"]
      directChildOnly: true
      sourceExtensions: [.tsx]
      excludeBasenames: [Internal.tsx]
      excludePrefixes: [_]
      companionGlobs: ["{sourceDir}/{sourceStem}.stories.tsx"]
      specifierTemplate: "@/components/{sourceStem}"
      stripSourcePrefix: src/
```

## Why and when

Use this rule when a source type must be accompanied by a test, story, or other
ownership file that imports it through one canonical specifier.

## What it catches/requires

Each selected source must have a matching companion file whose import satisfies
the configured `specifierTemplate`.

## Options and defaults

`sourceDirs` and `sourceGlobs` select source files. `directChildOnly` defaults
to `false`; `sourceExtensions` defaults to the built-in TypeScript/JavaScript
extensions. `excludeBasenames` and `excludePrefixes` default to empty lists.
`companionGlobs` renders expected companion paths, while `specifierTemplate`
renders the required import. `stripSourcePrefix` defaults to an empty string.
Empty `companionGlobs` or `specifierTemplate` disables findings. Templates may
use `{sourceDir}` and `{sourceStem}`.

## Valid example

```tsx
// Button.stories.tsx
import Button from "@/components/Button";
```

## Counterexample

`Button.stories.tsx` exists but imports `./Button` while the configured
`specifierTemplate` requires `@/components/Button`.

## Fix

Make the companion import match the rendered template or narrow the source and
companion globs to the intended ownership boundary.

## Suppression

Findings are reported on the source at line 1. Prefer an exclusion selector;
use `no-mistakes-disable-file required-companion-imports` for a documented
generated or externally owned file.

## Related rules

[`require-storybook-stories`](require-storybook-stories.md) checks component
story reachability; [`required-local-docs`](required-local-docs.md) checks the
presence of local documentation.
