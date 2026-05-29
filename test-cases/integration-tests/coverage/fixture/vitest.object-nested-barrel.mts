import { defineConfig } from 'vitest/config'
import { webTest } from './vitest.object-nested'

export default defineConfig({
  test: {
    projects: [
      {
        test: webTest,
      },
    ],
  },
})
