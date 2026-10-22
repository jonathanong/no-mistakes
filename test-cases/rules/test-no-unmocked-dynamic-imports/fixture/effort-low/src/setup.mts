import './static-helper.mts'
import './require-bridge.cjs'
import { vi } from 'vitest'

// Nested vi.mock stays accepted coverage at both efforts.
function nested() {
  vi.mock('./nested-target.mts', () => ({ value: 'nested' }))
}

vi.mock('./direct-target.mts', () => ({ value: 'direct' }))
// A non-literal specifier is uncredited at effort max and unanalyzable at low.
const dynamicName = './dynamic-name.mts'
vi.mock(dynamicName) // EXPECT_SHAPE
jest.mock(dynamicName) // EXPECT_SHAPE
// Aliased, computed, module.require, and import-equals forms stay uncredited
// at effort max and name the rewrite at effort low.
const { mock } = vi
mock('./direct-target.mts') // EXPECT_SHAPE
vi['mock']('./nested-target.mts') // EXPECT_SHAPE
module.require('./require-helper.cjs') // EXPECT_SHAPE
const requireAlias = require
requireAlias('./require-helper.cjs') // EXPECT_SHAPE
import equalsTarget = require('./require-helper.cjs') // EXPECT_SHAPE
// vi.doMock is coverage at effort max and unanalyzable at effort low.
vi.doMock('./do-target.mts', () => ({ value: 'do' })) // EXPECT_SHAPE
import('./dynamic-helper.mts') // EXPECT_SHAPE
jest.setMock('./set-target.mts', { value: 'set' }) // EXPECT_SHAPE
jest.unstable_mockModule('./unstable-target.mts', () => ({ value: 'unstable' })) // EXPECT_SHAPE
require.resolve('./resolved-target.mts') // EXPECT_SILENT
// no-mistakes-disable-next-line test-no-unmocked-dynamic-imports
import('./suppressed-dynamic.mts')
