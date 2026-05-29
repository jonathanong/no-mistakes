import { defineConfig } from 'vitest/config'
import { groups } from './vitest.member-namespace-star-missing-barrel'

// The barrel exports * as groups from a missing package.
// When resolving groups.web via imported_options_from_base, the resolver fails → line 87.
export default defineConfig({
  test: {
    // @ts-ignore
    projects: groups.web,
  },
})
