import { test, vi } from 'vitest'

// Counterintuitive on purpose: the first-argument `import('@lib/typed-covered.mts')` is a
// type carrier and must NOT be flagged. The `import('@lib/typed-factory-leaf.mts')` inside
// the factory body is a genuine dynamic import and IS unmocked, so it must still be flagged.
// Do not "simplify" this fixture by removing the factory-body import — it protects the
// invariant that mock detection does not swallow real dynamic imports inside the factory.
vi.mock(import('@lib/typed-covered.mts'), () => import('@lib/typed-factory-leaf.mts'))

test('typed mock carrier is not flagged, but the factory import still is', () => {
  // no-op: coverage is asserted by the rule, not runtime behavior
})
