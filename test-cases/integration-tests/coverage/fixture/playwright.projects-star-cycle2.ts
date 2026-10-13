import { defineConfig } from '@playwright/test'
// @ts-ignore
import { nonExistent } from './playwright.projects-star-cycle2-a'

// cycle2-a and cycle2-b form a mutual star-export cycle with no named exports.
// Looking for 'nonExistent' traverses A→B→A and resolves empty.
export default defineConfig({
  // @ts-ignore
  projects: nonExistent,
})
