import { defineConfig } from '@playwright/test'
import { starBarrelConfig } from './playwright.root-spread-star-barrel'

// starBarrelConfig is found via export* chain in the barrel
// Covers star barrel path in root_spreads.rs exported_project_options (line 164)
export default defineConfig({
  ...starBarrelConfig,
})
