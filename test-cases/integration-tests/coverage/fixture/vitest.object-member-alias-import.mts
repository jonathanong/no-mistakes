import { defineConfig } from 'vitest/config'
import { configs } from './vitest.object-member-alias-barrel'

// configs.web member access - barrel has export* as configs from missing
// Covers imported_options_from_base resolver failure in members.rs (line 87)
export default defineConfig({
  test: {
    projects: [configs.web],
  },
})
