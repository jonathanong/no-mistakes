import { defineConfig } from 'vitest/config'
import defaultObjectProject from './vitest.default-object'
import { reexportedProjects } from './vitest.projects-source'

const localProject = {
  test: {
    name: 'local-identifier-projects',
    include: ['local-identifier-projects/**/*.test.ts'],
  },
}

export default defineConfig({
  test: {
    projects: [localProject, defaultObjectProject, ...reexportedProjects],
  },
})

export const unusedProjects = [localProject]
