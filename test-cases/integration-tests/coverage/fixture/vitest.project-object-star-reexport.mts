import { defineConfig } from 'vitest/config'
import { projectDefaults } from './vitest.project-object-star-reexport-barrel'

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
