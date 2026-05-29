import { defineConfig } from 'vitest/config'
import shared from './vitest.root-test-override-clears-helper'

export default defineConfig({
  ...shared,
  test: {
    include: ['vitest-root-test-override-clears/**/*.test.ts'],
  },
})
