import { defineConfig } from 'vitest/config'
// @ts-ignore
import { importedConfigs } from './vitest.member-spread-member-unreadable-source'

// importedConfigs is from a directory (unreadable). spread member access exercises
// imported_member_options_from read error in members.rs (line 124).
const merged = { ...importedConfigs }
export default defineConfig({
  test: {
    // @ts-ignore
    projects: merged.web,
  },
})
