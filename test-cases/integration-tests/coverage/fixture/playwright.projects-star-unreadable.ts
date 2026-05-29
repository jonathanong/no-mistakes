import { defineConfig } from '@playwright/test'
// @ts-ignore
import { unreadableProjects } from './playwright.projects-star-unreadable-barrel'

// The barrel re-exports a directory (unreadable file) which exercises
// the Err(_) => Ok(None) path in imported_options_lookup (line 107 of exports.rs)
export default defineConfig({
  // @ts-ignore
  projects: unreadableProjects,
})
