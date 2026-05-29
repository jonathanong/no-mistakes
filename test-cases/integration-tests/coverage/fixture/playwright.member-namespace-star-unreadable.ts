import { defineConfig } from '@playwright/test'
import { configs } from './playwright.member-namespace-star-unreadable-barrel'

// The barrel exports * as configs from a directory (unreadable as file).
// When resolving configs.web via imported_options_from_base, the file read fails → line 101.
export default defineConfig({
  projects: [
    // @ts-ignore
    configs.web,
  ],
})
