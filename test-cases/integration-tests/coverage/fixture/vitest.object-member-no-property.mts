import { defineConfig } from 'vitest/config'
import { configs } from './vitest.object-member-no-property-source'

// Access a property that doesn't exist in configs
// Covers property_expression_deep None in members.rs (line 184)
export default defineConfig({
  test: {
    projects: [configs.nonExistent],
  },
})
