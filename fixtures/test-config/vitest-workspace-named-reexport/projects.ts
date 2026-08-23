import { defineWorkspace } from 'vitest/config'

export const projects = defineWorkspace([
  {
    test: {
      name: 'named-reexport-project',
      include: ['named/**/*.test.ts'],
      setupFiles: './workspace-setup.ts',
    },
  },
])
