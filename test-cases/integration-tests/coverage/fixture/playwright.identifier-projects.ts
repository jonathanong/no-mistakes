import { defineConfig } from '@playwright/test'
import { importedPlaywrightProjects } from './playwright.projects-helper'

const localProjects = [
  {
    name: 'local-identifier-projects',
    testMatch: ['local-identifier-projects/**/*.spec.ts'],
  },
]

export default defineConfig({
  projects: importedPlaywrightProjects,
})

export const unusedProjects = localProjects
