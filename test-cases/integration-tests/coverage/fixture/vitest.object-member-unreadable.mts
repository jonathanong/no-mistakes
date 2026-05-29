import { defineConfig } from 'vitest/config'
// @ts-ignore
import { configs } from './vitest.object-member-unreadable-source'

// configs is imported from a directory (unreadable as file).
// Accessing configs.web covers the Err(_) path in objects/members.rs (line 60).
export default defineConfig({
  test: {
    projects: [
      // @ts-ignore
      configs.web,
    ],
  },
})
