import { defineConfig } from 'vitest/config'
// @ts-ignore
import { nonExistent } from './vitest.projects-star-cycle-a'

// cycle-a and cycle-b form a mutual star-export cycle.
// When looking for 'nonExistent', the star loop traverses A→B→A, hitting
// cycle detection in imported_options_lookup (line 103 of exports.rs).
export default defineConfig({
  test: {
    // @ts-ignore
    projects: nonExistent,
  },
})
