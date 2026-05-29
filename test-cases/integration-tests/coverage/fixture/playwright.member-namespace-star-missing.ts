import { defineConfig } from '@playwright/test'
import { configs } from './playwright.member-namespace-star-missing-barrel'

// The barrel exports * as configs from a missing package.
// When resolving configs.web via imported_options_from_base, the resolver fails → line 95.
export default defineConfig({
  projects: [
    // @ts-ignore
    configs.web,
  ],
})
