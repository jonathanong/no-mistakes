import { defineConfig } from 'vitest/config'

// Project with exclude property inside test block.
// Exercises merge_property with name="exclude" in objects.rs (line 82).
export default defineConfig({
  test: {
    projects: [
      {
        test: {
          name: 'vitest-project-exclude',
          include: ['vitest-project-exclude/**/*.test.ts'],
          exclude: ['vitest-project-exclude/**/*.skip.ts'],
        },
      },
    ],
  },
})
