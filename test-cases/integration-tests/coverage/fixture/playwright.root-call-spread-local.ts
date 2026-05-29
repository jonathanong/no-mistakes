import { defineConfig } from '@playwright/test'

function makeConfig() {
  const projects = [
    { name: 'pw-root-call-local', testMatch: ['pw-root-call-local/**/*.spec.ts'] },
  ]
  return { projects }
}

export default defineConfig({ ...makeConfig() })
