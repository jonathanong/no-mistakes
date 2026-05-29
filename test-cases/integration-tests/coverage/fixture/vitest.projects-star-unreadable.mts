import { defineConfig } from 'vitest/config'
import { unreadableProjects } from './vitest.projects-star-unreadable-barrel'

// The barrel re-exports a directory (unreadable file) which exercises
// the Err(_) => Ok(None) path in imported_options_lookup (line 106)
export default defineConfig({
  test: {
    // @ts-ignore
    projects: unreadableProjects,
  },
})
