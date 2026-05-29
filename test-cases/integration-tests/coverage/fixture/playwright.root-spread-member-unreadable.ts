import { defineConfig } from '@playwright/test'
// @ts-ignore
import { config } from './playwright.root-spread-member-unreadable-source'

// The helper is a directory (unreadable as file), covering the Err(_) path
// in root_spreads/members.rs imported_member_project_options (line 46).
export default defineConfig({
  // @ts-ignore
  ...config.testConfig,
})
