import { test, vi } from 'vitest'

// Regression for #506: a typed mock's `import(...)` carrier is a real ImportExpression AST
// node, so the shared dependency graph records a dynamic-import edge from this test file to
// the mocked leaf even though the test never dynamically imports it itself. Reachability
// scanning must not treat the mocked leaf's own (unmocked) internal dynamic import as
// reachable and reportable — the leaf is fully replaced by the factory and never executes.
vi.mock(import('@lib/typed-mock-reachable-leaf.mts'), () => ({
  run: () => 'mocked',
}))

test('typed-only mock does not leak the mocked leaf into reachability scanning', () => {})
