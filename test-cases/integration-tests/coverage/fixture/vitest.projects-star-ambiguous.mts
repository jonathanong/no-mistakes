import { defineConfig } from 'vitest/config'
import { myStarProjects } from './vitest.projects-star-ambiguous-barrel'

// myStarProjects exists in two star sources - ambiguous, returns None
// Multiple matching star exports are ambiguous and resolve empty.
export default defineConfig({
  test: {
    projects: myStarProjects,
  },
})
