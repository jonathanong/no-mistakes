import { defineConfig } from 'vitest/config'
import { vitestProjects } from './vitest.projects'

export default defineConfig({
  test: {
    projects: vitestProjects,
  },
})
