# `integration-test-no-mocks`

## Why and when

Use this rule when an integration-test directory is expected to exercise real
boundaries instead of a unit-test substitute.

## What it catches

It catches configured mock-library imports in selected integration tests while
retaining explicit allowlists.

## Options

`forbiddenModules` is the only rule-local option. When it is empty, the rule
uses its built-in defaults: `msw`, `nock`, and `sinon`. Shared rule
`include`/`exclude` filters and the selected test project determine the files.
The removed `forbiddenCalls` option is rejected: move mock-invocation bans such
as `vi.mock` and `vi.spyOn` to a [`forbidden-calls`](forbidden-calls.md)
application.

## Valid example

An integration test that calls its real database/API boundary without a mocked
module passes.

```ts
// tests/integration/users.test.ts
import { db } from '../db';
test('loads', async () => { await db.query('select 1'); });
```


## Counterexample

```ts
// tests/integration/users.test.ts
import { setupServer } from 'msw/node';
```

## Analysis

This rule's import checks retain their own configured file demand.
Integration annotation policies under `tests.*.projects.*.integration_suites`
independently [scope recursive test discovery](../cli/check.md#integration-annotation-discovery);
that scope does not filter the import facts needed by this rule.

### Effort

- Runs at `max` only. Configuring `effort: low` (on the rule, or as a repository or CLI default) is not an error: `check` records the warning `requested effort low; effective effort max because this rule has no low implementation` and runs the rule at `max`.

### Unanalyzable inputs

The rule matches literal module specifiers in `from`, `import`, `require(`,
and dynamic `import(`. It skips unreadable files and does not resolve
computed specifiers. It does not use `options.unanalyzable`. Configure
call bans on `forbidden-calls`. [Source scanning](https://github.com/jonathanong/no-mistakes/blob/cc505a7c1e9c3cb6ad6d69cce53633b67d0c999e/crates/no-mistakes/src/codebase/rules/integration_test_no_mocks.rs#L139-L164) and [module patterns](https://github.com/jonathanong/no-mistakes/blob/cc505a7c1e9c3cb6ad6d69cce53633b67d0c999e/crates/no-mistakes/src/codebase/rules/integration_test_no_mocks/patterns.rs#L1-L6).

## Related rules

[`module-mock-boundary`](../eslint-rules/module-mock-boundary.md) governs
file-local ESLint mock policy; [`vitest-mock-test-file-naming`](../eslint-rules/vitest-mock-test-file-naming.md)
labels module-mocking tests. [`forbidden-calls`](forbidden-calls.md) bans
binding-aware mock helper invocations such as `vi.mock()` and `vi.spyOn()`.

Bans configured mocking libraries in integration tests.

```yaml
rules:
  - rule: integration-test-no-mocks
    projects: [web]
    include:
      - integration-tests/**/*.test.*
      - integration-tests/**/*.spec.*
    options:
      forbiddenModules: [msw, nock, sinon]
```

Counterexample: an integration test imports `msw/node` to replace production
behavior with a mock.

Fix: use the real dependency, move the behavior behind a test helper, or narrow
`forbiddenModules` so the rule matches your integration boundary.

Suppression caveat: suppress only a specific line when the mock is intentional
and unavoidable. Prefer tightening the rule config or moving the exception into
a dedicated helper instead of disabling the whole file.

## Invalid UTF-8

Invalid UTF-8 is decoded lossily and scanned, so a violation next to a stray byte is still reported. A file that cannot be read for any other reason is skipped and produces no finding.
