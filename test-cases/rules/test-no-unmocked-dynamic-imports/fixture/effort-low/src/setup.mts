import './static-helper.mts'
import './require-bridge.cjs'
import { vi } from 'vitest'

// Nested vi.mock stays accepted coverage at both efforts.
function nested() {
  vi.mock('./nested-target.mts', () => ({ value: 'nested' }))
}

vi.mock('./direct-target.mts', () => ({ value: 'direct' }))
// vi.doMock is coverage at effort max and unanalyzable at effort low.
vi.doMock('./do-target.mts', () => ({ value: 'do' })) // EXPECT_SHAPE
import('./dynamic-helper.mts') // EXPECT_SHAPE
jest.setMock('./set-target.mts', { value: 'set' }) // EXPECT_SHAPE
jest.unstable_mockModule('./unstable-target.mts', () => ({ value: 'unstable' })) // EXPECT_SHAPE
require.resolve('./resolved-target.mts') // EXPECT_SILENT
// no-mistakes-disable-next-line test-no-unmocked-dynamic-imports
import('./suppressed-dynamic.mts')
