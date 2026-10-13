import { defineConfig } from '@playwright/test'
import { myStarProjects } from './playwright.projects-star-barrel'

// myStarProjects found via export* chain
// A project array resolves through an export-star barrel.
export default defineConfig({
  projects: myStarProjects,
})
