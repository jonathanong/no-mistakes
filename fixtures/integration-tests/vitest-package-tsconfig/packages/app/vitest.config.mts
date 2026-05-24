import { defineConfig } from 'vitest/config'
import { packageProjects } from '@test-config/projects'

export default defineConfig({
  test: {
    projects: [...packageProjects],
  },
})
