import { expect, test } from 'vitest'
import { registerAliases } from '../shared/catalog.mts'

test('registers aliases', () => {
  expect(registerAliases()).toBeTruthy()
})
