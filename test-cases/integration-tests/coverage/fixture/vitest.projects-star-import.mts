import { defineConfig } from 'vitest/config'
import { myStarProjects } from './vitest.projects-star-barrel'

// A project array resolves through an export-star barrel.
export default defineConfig({
  test: {
    projects: myStarProjects,
  },
})
