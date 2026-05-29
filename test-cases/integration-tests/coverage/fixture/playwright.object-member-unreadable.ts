import { defineConfig } from '@playwright/test'
// @ts-ignore
import { configs } from './playwright.object-member-unreadable-source'

// configs is imported from a directory (unreadable as file).
// Accessing configs.web covers the Err(_) path in objects/members.rs (line 60).
export default defineConfig({
  projects: [
    // @ts-ignore
    configs.web,
  ],
})
