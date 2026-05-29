import { defineConfig } from '@playwright/test'
import { configs } from '@no-mistakes-test-nonexistent'

// Named import from missing package with member access
// Covers resolver failure in objects/members.rs imported_member_options (line 57)
export default defineConfig({
  projects: [configs.web],
})
