import { defineConfig } from 'vitest/config'
import { backendCoreProjects } from './test-helpers/vitest-config/backend-core-projects.mts'
import { toolingProjects } from './test-helpers/vitest-config/tooling-projects.mts'
import { webProjects } from './test-helpers/vitest-config/web-projects.mts'

export default defineConfig({
  test: {
    projects: [
      ...backendCoreProjects('react'),
      ...toolingProjects,
      ...webProjects({ root: 'web' }),
    ],
  },
})
