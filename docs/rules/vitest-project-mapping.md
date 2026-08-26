# `vitest-project-mapping`

Requires every selected Vitest test file to map to exactly one statically
configured Vitest project.

```yaml
tests:
  vitest:
    configs: vitest.config.mts

rules:
  - rule: vitest-project-mapping
    scope: repository
    options:
      testExtensions: [.test.ts, .test.tsx]
```

Counterexample: a test file matches no Vitest project include glob, or it
matches multiple project include globs after excludes are applied.

Fix: update the Vitest project `include` / `exclude` config so each test file
has one owner, or narrow this rule with `scopes`.

For Vitest configs that build `test.projects` through runtime-only helper calls,
declare project ownership explicitly and opt the rule into those declarations:

```yaml
tests:
  vitest:
    configs: vitest.config.mts
    projects:
      backend:
        include: [backend/**/*.test.ts]
      web:
        include: [web/**/*.test.ts]
        exclude: [web/**/*.generated.test.ts]

rules:
  - rule: vitest-project-mapping
    scope: repository
    options:
      explicitProjectsOnly: true
```

`explicitProjectsOnly` skips static parsing of `tests.vitest.configs` for this
rule and uses only `tests.vitest.projects`. This avoids false positives when the
config contains patterns such as `...makeProjects(args)`, which no-mistakes does
not execute.

## Why and when

Use this rule when each Vitest test needs one owning project for deterministic
commands, environment setup, and CI selection.

## What it catches/requires

Every selected test must match exactly one project include/exclude policy after
normalization; matching none or several projects is a finding.

## Options and defaults

`testExtensions` defaults to supported Vitest test extensions. Set
`explicitProjectsOnly: true` to use only `tests.vitest.projects` and skip
dynamic config parsing.

## Valid example

```yaml
projects:
  backend: {include: [backend/**/*.test.ts]}
  web: {include: [web/**/*.test.ts]}
```

## Counterexample

`web/profile.test.ts` matches both `web/**/*.test.ts` and a broad
`**/*.test.ts` project.

## Fix

Make project globs mutually exclusive, add the missing include, or enable
`explicitProjectsOnly` for a runtime-generated config with explicit ownership.

## Suppression

Prefer correcting project ownership. Use a file directive only for a generated
test artifact that should not be selected by Vitest.

## Related rules

[`vitest-test-correspondence`](vitest-test-correspondence.md) checks source/test
pairing; [`vitest-ci-path-coverage`](vitest-ci-path-coverage.md) checks CI path
filters for the projects.
