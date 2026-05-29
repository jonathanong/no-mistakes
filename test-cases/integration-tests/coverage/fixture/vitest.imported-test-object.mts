import { defineConfig } from 'vitest/config'
import { testConfig } from './vitest.imported-test-object-helper'

export default defineConfig({
  test: testConfig,
})
