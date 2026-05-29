import { defineConfig } from '@playwright/test'
import { shared } from '@no-mistakes-test-nonexistent'

// Named import from missing package, then spread and member access
// Covers imported_member_options_from resolver failure in project_arrays/members.rs (line 118)
const merged = { ...shared }

export default defineConfig({
  projects: merged.web,
})
