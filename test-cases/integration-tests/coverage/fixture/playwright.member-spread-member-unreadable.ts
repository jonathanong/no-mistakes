import { defineConfig } from '@playwright/test'
// @ts-ignore
import { importedConfigs } from './playwright.member-spread-member-unreadable-source'

// importedConfigs is from a directory (unreadable). spread member access exercises
// imported_member_options_from read error in members.rs (line 124).
const merged = { ...importedConfigs }
export default defineConfig({
  // @ts-ignore
  projects: merged.web,
})
