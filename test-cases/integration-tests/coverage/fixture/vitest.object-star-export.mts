import { defineConfig } from 'vitest/config'
import { objectStarConfig } from './vitest.object-star-export-barrel'

// The barrel has export type * and export * as X (skipped) + export * (resolved)
export default defineConfig({
  test: {
    projects: [objectStarConfig],
  },
})
