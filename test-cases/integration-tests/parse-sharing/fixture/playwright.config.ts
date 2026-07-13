import { defineConfig } from '@playwright/test'
import { playwrightProjects } from './vitest.projects'

export default defineConfig({
  projects: playwrightProjects,
})
