import { defineConfig } from 'vitest/config'
import { importedProjectTestBlock } from './vitest.imported-project-test-block-helper'

export default defineConfig({
  test: {
    projects: [
      {
        test: importedProjectTestBlock,
      },
    ],
  },
})
