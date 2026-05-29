import { defineConfig } from '@playwright/test'
import { myStarProjects } from './playwright.projects-star-barrel'

// myStarProjects found via export* chain
// Covers imported_options_lookup success path in exports.rs (line 107)
export default defineConfig({
  projects: myStarProjects,
})
