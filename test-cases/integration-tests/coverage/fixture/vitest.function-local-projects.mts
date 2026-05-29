import { defineConfig } from 'vitest/config'

function makeProjects() {
  const projects = [
    {
      test: {
        name: 'vitest-function-local-projects',
        include: ['vitest-function-local-projects/**/*.test.ts'],
      },
    },
  ]
  return projects
}

export default defineConfig({
  test: {
    projects: makeProjects(),
  },
})
