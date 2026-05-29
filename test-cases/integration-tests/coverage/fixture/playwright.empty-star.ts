import { defineConfig } from '@playwright/test'
import { emptyStarProjects } from './playwright.empty-star-barrel'

export default defineConfig({
  projects: emptyStarProjects,
})
