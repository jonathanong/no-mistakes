import { defineWorkspace } from 'vitest/config'

export default defineWorkspace([
  {
    test: {
      name: 'workspace-project',
      include: ['workspace/**/*.test.ts'],
      setupFiles: './workspace-setup.ts',
    },
  },
])
