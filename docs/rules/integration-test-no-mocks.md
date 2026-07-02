# `integration-test-no-mocks`

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
