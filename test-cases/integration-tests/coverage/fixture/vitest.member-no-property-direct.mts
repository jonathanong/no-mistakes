import { defineConfig } from 'vitest/config'
import { configs } from './vitest.object-member-no-property-source'

// Access a non-existent property via direct projects assignment (not array).
// This exercises members.rs::exported_member_options where property_expression_deep
// returns None (line 184).
export default defineConfig({
  test: {
    // @ts-ignore
    projects: configs.nonExistent,
  },
})
