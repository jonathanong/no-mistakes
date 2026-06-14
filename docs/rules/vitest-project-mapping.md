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
