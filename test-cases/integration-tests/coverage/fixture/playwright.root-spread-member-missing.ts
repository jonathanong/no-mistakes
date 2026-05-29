import { defineConfig } from '@playwright/test'
import { configs } from '@no-mistakes-test-nonexistent'

// Spread object with named import from missing package
// Covers resolver failure in root_spreads/members.rs imported_member_project_options (line 43)
const merged = { ...configs }

export default defineConfig({
  ...merged.web,
})
