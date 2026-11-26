<!-- cspell:ignore dorny -->
# `vitest-ci-path-coverage`

Requires Vitest project inputs to be covered by GitHub Actions path filters.

```yaml
projects:
  ts-shared:
    root: ts-shared

tests:
  vitest:
    projects:
      backend:
        include: [backend/**/*.test.ts]
      shared:
        include: ["ts-shared/**/*.test.ts"]

testPlan:
  vitest:
    fullSuiteTriggers:
      projects:
        ts-shared:
          paths:
            - ts-shared/**
            - "!ts-shared/generated/**"
            - ts-shared/generated/schema.ts
          targets: [backend, shared]

rules:
  - rule: vitest-ci-path-coverage
    scope: repository
    options:
      projectFilters:
        backend: [backend]
        shared: [backend]
      workflows:
        - path: .github/workflows/ci.yml
          job: detect-changes
          stepId: filter
```

The rule reads Vitest project `include` / `exclude` globs, plus
`testPlan.vitest.fullSuiteTriggers.projects`, and checks visible files selected
by those globs against path-filter action `with.filters` maps in workflow
steps. Visible files include untracked files outside ignored paths. [File discovery](https://github.com/jonathanong/no-mistakes/blob/cc505a7c1e9c3cb6ad6d69cce53633b67d0c999e/crates/no-mistakes/src/codebase/ts_source/discovery/visible.rs#L1-L14) and [coverage scan](https://github.com/jonathanong/no-mistakes/blob/cc505a7c1e9c3cb6ad6d69cce53633b67d0c999e/crates/no-mistakes/src/codebase/rules/vitest_ci_path_coverage/scan.rs#L135-L145).

For target-scoped `{ paths, targets }` triggers, the rule attributes each
positive trigger path to the named Vitest runner projects. Ordered `!` paths
remove earlier matches and later positive paths may re-include them. Test-plan
preparation validates target names against its prepared runner-project catalog.

Legacy broad boolean and path-list triggers are also checked:

```yaml
projects:
  shared: { root: . }
  generated: { root: . }
testPlan:
  vitest:
    fullSuiteTriggers:
      projects:
        shared: true
        generated: [generated/**, "!generated/fixtures/**"]
```

Counterexample: the target-scoped trigger covers `ts-shared/**`, but the
`backend` CI filter only contains `ts-shared/*/package.json`. A change to
`ts-shared/utils/index.mts` would select both Vitest projects locally without
triggering CI.

Fix: broaden the workflow path filter, narrow the configured Vitest input
globs, or map the project to the correct filter names with `projectFilters`.

Options:

- `projectFilters` maps Vitest project names to CI filter names. Defaults to
  the project name.
- `sourceGlobsByProject` adds explicit project source/input globs.
- `includeVitestProjectGlobs` defaults to `true`.
- `includeFullSuiteTriggers` defaults to `true`.
- `explicitProjectsOnly` uses only `tests.vitest.projects` declarations.
- `workflows` optionally narrows parsing to selected workflow `path`, `job`,
  and `stepId`.

Suppression caveat: findings report line 1 of the workflow file, so the
practical opt-out is a top-of-file `no-mistakes-disable-file
vitest-ci-path-coverage` directive. Prefer fixing the filter unless coverage is
intentionally enforced elsewhere.

## Why and when

Use this rule when local Vitest selection is narrower than CI's path-filter
triggers and a changed source file could skip the required test job.

## What it catches/requires

Each configured Vitest project input must be covered by the mapped workflow
filter, including ordered negations and re-inclusions. Target-scoped full-suite
triggers are checked against their named runner projects.

## Options and defaults

`projectFilters` defaults each Vitest project to its own filter name;
`sourceGlobsByProject`, `workflows`, and `explicitProjectsOnly` are optional;
`includeVitestProjectGlobs` and `includeFullSuiteTriggers` default to `true`.

## Valid example

```yaml
# .no-mistakes.yml
projects:
  ts-shared: {root: ts-shared}
tests:
  vitest:
    projects:
      shared:
        include: [ts-shared/**/*.test.ts]
testPlan:
  vitest:
    fullSuiteTriggers:
      projects:
        ts-shared:
          paths: ["ts-shared/**"]
          targets: [shared]
rules:
  - rule: vitest-ci-path-coverage
    scope: repository
    options:
      projectFilters:
        shared: [shared]
      sourceGlobsByProject:
        shared: ['ts-shared/**/*.mts']
      workflows:
        - path: .github/workflows/ci.yml
          job: detect-changes
          stepId: filter
```

```yaml
# .github/workflows/ci.yml
name: CI
on: pull_request
jobs:
  detect-changes:
    runs-on: ubuntu-latest
    steps:
      - uses: dorny/paths-filter@v3
        id: filter
        with:
          filters: |
            shared:
              - 'ts-shared/**'
```

```ts
// ts-shared/utils/index.test.ts
import { it } from 'vitest';
import { add } from './index.mts';

it('adds values', () => add(1, 1));
```

```ts
// ts-shared/utils/index.mts
export const add = (left: number, right: number) => left + right;
```

The filter covers `ts-shared/**/*.test.ts` and its source inputs.

## Counterexample

```yaml
# .no-mistakes.yml
projects:
  ts-shared: {root: ts-shared}
tests:
  vitest:
    projects:
      shared:
        include: ["ts-shared/**/*.test.ts"]
testPlan:
  vitest:
    fullSuiteTriggers:
      projects:
        ts-shared:
          paths: ["ts-shared/**"]
          targets: [shared]
rules:
  - rule: vitest-ci-path-coverage
    scope: repository
    options:
      projectFilters:
        shared: [shared]
      sourceGlobsByProject:
        shared: ['ts-shared/**/*.mts']
      workflows:
        - path: .github/workflows/ci.yml
          job: detect-changes
          stepId: filter
```

```yaml
# .github/workflows/ci.yml
name: CI
on: pull_request
jobs:
  detect-changes:
    runs-on: ubuntu-latest
    steps:
      - uses: dorny/paths-filter@v3
        id: filter
        with:
          filters: |
            shared:
              - 'ts-shared/*/package.json'
```

```ts
// ts-shared/utils/index.test.ts
import { it } from 'vitest';
import { add } from './index.mts';

it('adds values', () => add(1, 1));
```

```ts
// ts-shared/utils/index.mts
export const add = (left: number, right: number) => left + right;
```

This misses a nested `ts-shared/utils/index.mts` change selected locally.

## Fix

Broaden the workflow filter, correct `projectFilters`, or narrow the Vitest
project input to the files CI really runs.

## Analysis

### Effort

- Runs at `max` only. Configuring `effort: low` (on the rule, or as a repository or CLI default) is not an error: `check` records the warning `requested effort low; effective effort max because this rule has no low implementation` and runs the rule at `max`.

### Unanalyzable inputs

Unreadable or malformed workflow YAML, unreadable or malformed external
filter YAML, and invalid filter globs produce ordinary findings. Filter
values outside the recognized forms are skipped. The rule does not use
`options.unanalyzable`. [Workflow reads](https://github.com/jonathanong/no-mistakes/blob/cc505a7c1e9c3cb6ad6d69cce53633b67d0c999e/crates/no-mistakes/src/codebase/rules/vitest_ci_path_coverage/workflow_filters.rs#L66-L89), [filter reads and glob validation](https://github.com/jonathanong/no-mistakes/blob/cc505a7c1e9c3cb6ad6d69cce53633b67d0c999e/crates/no-mistakes/src/codebase/rules/vitest_ci_path_coverage/workflow_filters/values.rs#L17-L57), and [filter selection](https://github.com/jonathanong/no-mistakes/blob/cc505a7c1e9c3cb6ad6d69cce53633b67d0c999e/crates/no-mistakes/src/codebase/rules/vitest_ci_path_coverage/workflow_filters/step.rs#L47-L62).

## Suppression

Findings point to workflow line 1, so use a top-of-file
`no-mistakes-disable-file vitest-ci-path-coverage` directive only when another
CI gate intentionally owns the coverage.

## Limitations

Discovery skips any directory segment named `node_modules`, `dist`, `.git`,
`.next`, `coverage`, `fixtures`, `target`, or `build`, so those files are not
compared with CI path filters. With a Vitest include of `src/**/*.test.ts`,
`src/build/widget.test.ts` is not reported.

## Related rules

[`vitest-project-mapping`](vitest-project-mapping.md) assigns each test to one
project; [`tsconfig-gate-coverage`](tsconfig-gate-coverage.md) performs the
analogous CI/local gate check for TypeScript configs.
