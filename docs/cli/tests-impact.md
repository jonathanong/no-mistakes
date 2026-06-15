# `no-mistakes tests impact`

Find impacted tests from explicit changed files.

```sh
no-mistakes tests impact src/api.mts --format json
```

Use this when an agent already knows the changed files and wants a structured
test set without parsing a git diff. Impact traversal is file-scoped today;
`file#symbol` inputs are accepted for compatibility but the symbol suffix does
not narrow the result.

Key options: `--root`, `--config`, `--tsconfig`, `--format`, and `--json`.

## Dynamic imports and `next/dynamic`

Impact traversal follows runtime `import()` boundaries, including
`next/dynamic(() => import('./Foo'))`: changing `Foo` surfaces the tests that
reach it through the dynamic caller, at `medium` confidence with a
`dynamic-import` warning. Computed specifiers (non-string-literal `import(...)`)
cannot be resolved statically and are not traversed.

## Stub tests and registry hints (opt-in)

Two [`tests.impact`](../configuration/tests.md#testsimpact) config knobs tune the
output:

- `alwaysIncludeTests` surfaces stub/mock tests (e.g. `**/*.mock.test.*`) that a
  suite `exclude` would otherwise drop, so you can update the stub before
  pushing.
- `registries` emits a `registry-hint` warning when the changed file is imported
  by a registry file (e.g. `auth-gated-code-splitting.mts`), reminding you to
  verify the registry entry.

Node API: `testsImpact(options)`. Both knobs are read from `.no-mistakes.yml`, so
the Node API honors them with no extra options.
