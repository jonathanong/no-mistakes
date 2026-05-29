import { defineConfig } from 'vitest/config'
import { emptyStarProjects } from './vitest.empty-star-barrel'

export default defineConfig({
  test: {
    projects: emptyStarProjects,
  },
})
