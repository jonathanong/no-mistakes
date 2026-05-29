import { namedImportedTestOptions } from './vitest.imported-test-spread-helper'
import { defineConfig } from 'vitest/config'

export default defineConfig({
  test: {
    ...namedImportedTestOptions,
  },
})
