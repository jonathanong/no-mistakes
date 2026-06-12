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
