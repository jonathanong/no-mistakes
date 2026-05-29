import { defineConfig } from 'vitest/config'
import { projectDefaults } from './vitest.project-object-sourced-reexport-barrel'

export default defineConfig({
  test: {
    projects: [
      {
        test: {
          ...projectDefaults,
        },
      },
    ],
  },
})
