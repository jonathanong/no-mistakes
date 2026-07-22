import { defineWorkspace } from 'vitest/config'

module.exports = defineWorkspace([{
  test: {
    name: 'commonjs-define-project',
    setupFiles: './workspace-setup.ts',
  },
}])
