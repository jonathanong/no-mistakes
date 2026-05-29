import { defineConfig } from '@playwright/test'
import { objectStarConfig } from './playwright.object-star-export-barrel'

// objectStarConfig comes from an ambiguous star barrel
// The ambiguous case returns null and the project is not found
export default defineConfig({
  projects: [objectStarConfig],
})
