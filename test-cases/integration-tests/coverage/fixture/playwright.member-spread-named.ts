import { defineConfig } from '@playwright/test'
import { shared } from './playwright.member-spread-named-source'

// Named import (not *) spread, then member access in projects array
// Covers imported_spread_member_options found = options path (line 70)
const merged = { ...shared }

export default defineConfig({
  projects: merged.web,
})
