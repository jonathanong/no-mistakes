import { defineConfig } from '@playwright/test'
import { myStarProjects } from './playwright.projects-star-cycle-a'

// A cyclic star barrel can still resolve its concrete named export.
export default defineConfig({
  projects: myStarProjects,
})
