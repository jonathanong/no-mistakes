import { defineConfig } from 'vitest/config'
import { myStarProjects } from './vitest.projects-star-barrel'

// Covers imported_options_lookup (resolver failure and success) in exports.rs
export default defineConfig({
  test: {
    projects: myStarProjects,
  },
})
