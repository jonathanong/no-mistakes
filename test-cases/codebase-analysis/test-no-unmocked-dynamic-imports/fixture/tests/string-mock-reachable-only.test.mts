import { test, vi } from 'vitest'

// Sibling case to typed-mock-reachable-only.test.mts: a string-literal mock specifier is
// not an ImportExpression, so it never creates a dynamic-import graph edge on its own. This
// pins that the mocked leaf's internals stay unreported for the string form too.
vi.mock('@lib/typed-mock-reachable-leaf.mts', () => ({
  run: () => 'mocked',
}))

test('string-only mock does not leak the mocked leaf into reachability scanning', () => {})
