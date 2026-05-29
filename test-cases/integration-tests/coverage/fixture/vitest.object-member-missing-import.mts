import { defineConfig } from 'vitest/config'
import { bases } from '@missing-no-mistakes-pkg'

export default defineConfig({
  test: {
    projects: [
      {
        ...bases.web,
        test: { name: 'vitest-object-member-missing-import' },
      },
    ],
  },
})
