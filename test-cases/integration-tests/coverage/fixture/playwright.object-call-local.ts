import { defineConfig } from '@playwright/test'

function makeProject() {
  const testMatch = ['pw-object-call-local/**/*.spec.ts']
  return {
    name: 'pw-object-call-local',
    testMatch,
  }
}

export default defineConfig({
  projects: [makeProject()],
})
