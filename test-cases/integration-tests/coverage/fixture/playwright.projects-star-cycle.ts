import { defineConfig } from '@playwright/test'
import { myStarProjects } from './playwright.projects-star-cycle-a'

// Covers cycle detection in imported_options_lookup (line 104)
export default defineConfig({
  projects: myStarProjects,
})
