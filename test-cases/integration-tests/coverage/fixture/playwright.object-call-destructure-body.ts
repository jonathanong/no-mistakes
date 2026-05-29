import { defineConfig } from '@playwright/test'

// Local function with destructuring in body covers function_body_bindings continue path
function makeProject() {
  const { dir } = { dir: 'pw-destructure-body' }
  return {
    name: 'pw-object-call-destructure-body',
    testMatch: ['pw-object-call-destructure-body/**/*.spec.ts'],
  }
}

export default defineConfig({
  projects: [makeProject()],
})
