import { defineConfig } from '@playwright/test'
import { sharedConfig } from './playwright.root-spread-named-member-source'

// Named import (not *) spread, then member access
// Covers imported_member_project_options path (line 126) in root_spreads/members.rs
const merged = { ...sharedConfig }

export default defineConfig({
  ...merged.web,
})
