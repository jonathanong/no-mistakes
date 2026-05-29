import { defineConfig } from 'vitest/config'
import { groups } from './vitest.member-namespace-star-unreadable-barrel'

// The barrel exports * as groups from a directory (unreadable as file).
// When resolving groups.web via imported_options_from_base, the file read fails → line 93.
export default defineConfig({
  test: {
    // @ts-ignore
    projects: groups.web,
  },
})
