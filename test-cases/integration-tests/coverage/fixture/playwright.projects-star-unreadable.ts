import { defineConfig } from '@playwright/test'
// @ts-ignore
import { unreadableProjects } from './playwright.projects-star-unreadable-barrel'

// The barrel re-exports a directory (unreadable file) which exercises
// An unreadable star re-export target contributes no projects.
export default defineConfig({
  // @ts-ignore
  projects: unreadableProjects,
})
