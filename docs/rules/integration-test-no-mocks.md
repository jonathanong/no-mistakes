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

### Effort

- Runs at `max` only. Configuring `effort: low` (on the rule, or as a repository or CLI default) is not an error: `check` records the warning `requested effort low; effective effort max because this rule has no low implementation` and runs the rule at `max`.

### Unanalyzable inputs

Following [recall over precision](../../README.md#how-analysis-works), nothing this rule cannot prove is treated as clean, and it has no such inputs.

- This rule has no unanalyzable inputs: it judges the mock calls and imports present in selected integration test files. Nothing is routed through `options.unanalyzable`.

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
