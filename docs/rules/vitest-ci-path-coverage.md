# `vitest-ci-path-coverage`

Requires Vitest project inputs to be covered by GitHub Actions path filters.

```yaml
projects:
  ts-shared:
    root: ts-shared

testPlan:
  vitest:
    fullSuiteTriggers:
      projects:
        ts-shared:
          - "**"

rules:
  - rule: vitest-ci-path-coverage
    scope: repository
    options:
      projectFilters:
        ts-shared: [backend]
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

Counterexample: a Vitest project has a full-suite trigger for `ts-shared/**`,
but the CI filter only contains `ts-shared/*/package.json`. Source changes such
as `ts-shared/utils/index.mts` would not trigger CI.

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

Suppression: use `no-mistakes` suppression directives. Findings report line 1
on the workflow file.
