import { defineConfig } from 'vitest/config'
import { sharedTestConfig } from './vitest.test-sourced-reexport-barrel'

export default defineConfig({
  test: {
    ...sharedTestConfig,
  },
})
