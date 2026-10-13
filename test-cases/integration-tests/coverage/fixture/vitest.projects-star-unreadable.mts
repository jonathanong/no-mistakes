import { defineConfig } from 'vitest/config'
import { unreadableProjects } from './vitest.projects-star-unreadable-barrel'

// The barrel re-exports a directory (unreadable file) which exercises
// An unreadable star re-export target contributes no projects.
export default defineConfig({
  test: {
    // @ts-ignore
    projects: unreadableProjects,
  },
})
