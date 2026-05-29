import { defineConfig } from 'vitest/config'
import { shared } from './vitest.member-spread-named-source'

// Named import (not *) spread, then member access in projects array
// Covers imported_spread_member_options found = options path (line 70)
const merged = { ...shared }

export default defineConfig({
  test: {
    projects: merged.web,
  },
})
