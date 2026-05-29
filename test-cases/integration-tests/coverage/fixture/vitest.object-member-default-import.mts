import { defineConfig } from 'vitest/config'
import defaultBases from './vitest.object-member-default-import-source'

export default defineConfig({
  test: {
    projects: [
      {
        ...defaultBases.web,
        test: { name: 'vitest-object-member-default-import' },
      },
    ],
  },
})
