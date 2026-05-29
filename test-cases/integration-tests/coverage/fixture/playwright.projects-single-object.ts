import { defineConfig } from '@playwright/test'

// Unusual: projects is a single object instead of an array
// The parser handles this gracefully and wraps it in a Vec
export default defineConfig({
  // @ts-ignore
  projects: { name: 'pw-single-object-project', testMatch: ['pw-single-object-project/**/*.spec.ts'] },
})
