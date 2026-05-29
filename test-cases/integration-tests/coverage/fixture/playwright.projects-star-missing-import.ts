import { defineConfig } from '@playwright/test'
import { myStarProjects } from './playwright.projects-star-missing-barrel'

// myStarProjects found after skipping missing package star export
// Covers resolver failure in imported_options_lookup (line 101)
export default defineConfig({
  projects: myStarProjects,
})
