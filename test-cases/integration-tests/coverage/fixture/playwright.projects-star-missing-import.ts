import { defineConfig } from '@playwright/test'
import { myStarProjects } from './playwright.projects-star-missing-barrel'

// myStarProjects found after skipping missing package star export
// An unresolved star re-export contributes no projects.
export default defineConfig({
  projects: myStarProjects,
})
