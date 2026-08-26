# `integration-test-no-mocks`

## Why and when

Use this rule when an integration-test directory is expected to exercise real
boundaries instead of a unit-test substitute.

## What it catches

It catches configured mock-library imports, mock helpers, and mock-like calls
in selected integration tests while retaining explicit allowlists.

## Options

`forbiddenCalls` and `forbiddenModules` are the only rule-local options. When
either list is empty, the rule uses its built-in defaults: Vitest/Jest mock and
spy helpers for calls, and `msw`, `nock`, and `sinon` for modules. Shared rule
`include`/`exclude` filters and the selected test project determine the files.

## Valid example

An integration test that calls its real database/API boundary without a mocked
module or helper passes.

## Related rules

[`module-mock-boundary`](../eslint-rules/module-mock-boundary.md) governs
file-local ESLint mock policy; [`vitest-mock-test-file-naming`](../eslint-rules/vitest-mock-test-file-naming.md)
labels module-mocking tests.

Bans configured mocking libraries and mock helpers in integration tests.

```yaml
rules:
  - rule: integration-test-no-mocks
    projects: [web]
    include:
      - integration-tests/**/*.test.*
      - integration-tests/**/*.spec.*
    options:
      forbiddenCalls: [vi.mock, vi.spyOn]
      forbiddenModules: [msw, nock, sinon]
```

Counterexample: an integration test calls `vi.mock()` or imports `msw/node` to
replace production behavior with a mock.

Fix: use the real dependency, move the behavior behind a test helper, or narrow
`forbiddenCalls` and `forbiddenModules` so the rule matches your integration
boundary.

Suppression caveat: suppress only a specific line when the mock is intentional
and unavoidable. Prefer tightening the rule config or moving the exception into
a dedicated helper instead of disabling the whole file.
