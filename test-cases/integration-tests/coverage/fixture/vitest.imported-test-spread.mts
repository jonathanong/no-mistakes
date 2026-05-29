import shared from './vitest.imported-test-spread-helper'
import { defineConfig } from 'vitest/config'

export default defineConfig({
  test: {
    ...shared,
  },
})
