import { defineConfig } from 'vitest/config'
// @ts-ignore
import { config } from './vitest.root-spread-member-unreadable-source'

// The helper is a directory (unreadable as file), covering the Err(_) path
// in root_spreads/members.rs imported_member_project_options (line 48).
export default defineConfig({
  // @ts-ignore
  ...config.testConfig,
})
