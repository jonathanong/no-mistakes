import { defineConfig } from '@playwright/test'
// @ts-ignore
import { nonExistent } from './playwright.projects-star-cycle2-a'

// cycle2-a and cycle2-b form a mutual star-export cycle with no named exports.
// When looking for 'nonExistent', the star loop traverses A→B→A, hitting
// cycle detection in imported_options_lookup (line 104 of exports.rs).
export default defineConfig({
  // @ts-ignore
  projects: nonExistent,
})
