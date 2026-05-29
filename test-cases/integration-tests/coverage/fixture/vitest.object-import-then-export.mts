import { defineConfig } from 'vitest/config'
import { webTest } from './vitest.object-import-then-export-barrel'

export default defineConfig({
  test: {
    projects: [
      {
        test: webTest,
      },
    ],
  },
})
