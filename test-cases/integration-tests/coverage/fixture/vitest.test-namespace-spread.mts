import { defineConfig } from 'vitest/config'
import * as shared from './vitest.root-namespace-spread-helper'

export default defineConfig({
  test: {
    ...shared.testConfig,
  },
})
