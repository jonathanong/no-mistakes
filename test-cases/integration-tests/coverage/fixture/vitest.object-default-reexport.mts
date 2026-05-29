import { defineConfig } from 'vitest/config'
import base from './vitest.object-default-reexport-barrel'

export default defineConfig({
  test: {
    projects: [
      {
        test: base,
      },
    ],
  },
})
