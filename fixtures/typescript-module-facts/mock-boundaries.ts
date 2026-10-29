import { vi as v, vitest as vt } from 'vitest'
import { jest as j } from '@jest/globals'
import type { vi as typeOnly } from 'vitest'
import { type vi as individualType } from 'vitest'
import defaultMock from 'vitest'
import * as namespaceMock from 'vitest'
import { vi as otherPackage } from 'other'

v.mock('function', function () { return { value: 1 } })
vt.mock('values', () => ({ nested: { ok: true }, list: [1, `text`, () => fakeResult], fn: function () { return fakeResult } }))
j.mock('jest-alias')
vi.mock('vi-global')
jest.mock('jest-global')
;(v.mock('wrapped') as unknown)
v.mock('construct', () => { new Date(); return {} })
v.mock('body-prefix', () => { const value = 1; Date.now(); return {} })
v.mock('params', (original) => ({}))
v.mock('rest', (...args) => ({}))
v.mock('generator', function* () { return {} })
v.mock('factory-name', replacement)
v.mock('extra-args', () => ({}), extra)
v.mock('empty', () => {})
v.mock('no-return', () => { Date.now() })
v.mock('returnless', function () { return })
v.mock('flow', () => { if (condition) { Date.now() }; return {} })
v.mock('named-result', () => replacement)
v.mock('object-spread', () => ({ ...original }))
v.mock('argument-spread', () => { invoke(...values); return {} })
v.mock('function-constructor', () => { new Function('return 1'); return {} })
v.mock('function-call', () => { Function('return 1'); return {} })
v.mock('eval', () => { eval('1'); return {} })
v.mock('actual', () => { v.importActual('./actual'); return {} })
v.mock('dynamic-import', () => { import('./actual'); return {} })
v.mock('template-hole', () => ({ value: `${original}` }))
v.mock('named-value', () => ({ value: original }))
v.mock('getter', () => ({ get value() { return 1 } }))
v.mock('computed', () => ({ [key]: 1 }))
v.mock('method', () => ({ method() {} }))
v.mock('prototype', () => ({ __proto__: {} }))
v.mock('optional-call')?.value
v.mock?.('optional-invocation')
v?.mock('optional-member')
v.mock()
function nested() { v.mock('nested') }
if (condition) v.mock('conditional')
typeof v.mock('typeof-call')
v.doMock('deferred', () => ({}))
v.unmock('unmock')
v.doUnmock('do-unmock')
v.mock(dynamicSpecifier, () => ({}))
typeOnly.mock('ignored-type-import')
individualType.mock('ignored-individual-type')
defaultMock.mock('ignored-default')
namespaceMock.mock('ignored-namespace')
otherPackage.mock('ignored-package')
unknownGlobal.mock('ignored-global')
getMock().mock('ignored-receiver')
function shadow(vi: unknown) { vi.mock('ignored-shadow') }
