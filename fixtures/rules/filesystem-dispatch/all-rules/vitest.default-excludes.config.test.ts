import { expect, test } from 'vitest'
import config from './vitest.default-excludes.config'

test('keeps the default Vitest settings', () => {
  expect(config).toEqual({})
})
