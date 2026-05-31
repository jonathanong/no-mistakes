# `no-mistakes/vitest-mock-test-file-naming`

Requires `.mock.test` filenames for mock-heavy tests.

Why: mock-heavy tests need obvious filenames so agents can reason about dynamic
imports, manual mocks, and test isolation.

Counterexample: `user.test.ts` containing extensive `vi.mock(...)` setup.

Fix: rename the test file to use the configured mock-test filename pattern.
