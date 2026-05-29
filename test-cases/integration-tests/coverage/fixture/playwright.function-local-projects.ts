import { defineConfig } from '@playwright/test'

function makeProjects() {
  const projects = [
    {
      name: 'pw-function-local-projects',
      testMatch: ['pw-function-local-projects/**/*.spec.ts'],
    },
  ]
  return projects
}

export default defineConfig({
  projects: makeProjects(),
})
