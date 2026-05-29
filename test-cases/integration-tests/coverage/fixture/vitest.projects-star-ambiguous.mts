import { defineConfig } from 'vitest/config'
import { myStarProjects } from './vitest.projects-star-ambiguous-barrel'

// myStarProjects exists in two star sources - ambiguous, returns None
// Covers ambiguity branch in imported_options_lookup (line 118)
export default defineConfig({
  test: {
    projects: myStarProjects,
  },
})
