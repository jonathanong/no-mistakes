import { defineConfig } from '@playwright/test'
import { configs } from './playwright.object-member-alias-barrel'

// configs.web member access - barrel has export* as configs from missing
// Covers imported_options_from_base resolver failure in members.rs (line 95)
export default defineConfig({
  projects: [configs.web],
})
