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
# .no-mistakes.yml
tests:
  vitest:
    projects:
      backend: {include: ["backend/**/*.test.ts"]}
      web: {include: ["web/**/*.test.ts"]}
rules:
  - rule: vitest-project-mapping
    scope: repository
    options:
      explicitProjectsOnly: true
```

```ts
// backend/api.test.ts
import { it } from 'vitest';

it('runs in the backend project', () => {});
```

```ts
// web/profile.test.ts
import { it } from 'vitest';

it('runs in the web project', () => {});
```

## Counterexample

`web/profile.test.ts` matches both `web/**/*.test.ts` and a broad
`**/*.test.ts` project.

```yaml
# .no-mistakes.yml
tests:
  vitest:
    projects:
      broad: {include: ['**/*.test.ts']}
      web: {include: ['web/**/*.test.ts']}
rules:
  - rule: vitest-project-mapping
    scope: repository
    options:
      explicitProjectsOnly: true
```

```ts
// web/profile.test.ts
import { it } from 'vitest';

it('has two matching projects', () => {});
```


## Fix

Make project globs mutually exclusive, add the missing include, or enable
`explicitProjectsOnly` for a runtime-generated config with explicit ownership.

## Analysis

### Effort

- Runs at `max` only. Configuring `effort: low` (on the rule, or as a repository or CLI default) is not an error: `check` records the warning `requested effort low; effective effort max because this rule has no low implementation` and runs the rule at `max`.

### Unanalyzable inputs

Following [recall over precision](../../README.md#how-analysis-works), nothing this rule cannot prove is treated as clean, and it has no such inputs.

- This rule has no unanalyzable inputs: it maps test files to configured Vitest projects by glob. Nothing is routed through `options.unanalyzable`.

## Suppression

Prefer correcting project ownership. Use a file directive only for a generated
test artifact that should not be selected by Vitest.

## Limitations

Discovery skips any directory segment named `node_modules`, `dist`, `.git`,
`.next`, `coverage`, `fixtures`, `target`, or `build`. A test under one of
those names is not visible, so it is not reported as unmapped or multiply
mapped.

```text
src/build/users.test.ts
```

## Related rules

[`vitest-test-correspondence`](vitest-test-correspondence.md) checks source/test
pairing; [`vitest-ci-path-coverage`](vitest-ci-path-coverage.md) checks CI path
filters for the projects.
