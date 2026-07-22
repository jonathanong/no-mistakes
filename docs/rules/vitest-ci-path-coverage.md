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
        include: [ts-shared/**/*.test.ts]

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
`testPlan.vitest.fullSuiteTriggers.projects`, and checks tracked files selected
by those globs against path-filter action `with.filters` maps in workflow
steps.

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
