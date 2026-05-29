import { defineConfig } from 'vitest/config'
import { sharedConfig } from './vitest.root-spread-named-member-source'

// Named import (not *) spread, then member access
// Covers imported_member_project_options path (line 139) in root_spreads/members.rs
const merged = { ...sharedConfig }

export default defineConfig({
  ...merged.web,
})
