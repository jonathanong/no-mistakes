import { defineWorkspace } from 'vitest/config'

export default defineWorkspace([
  {
    name: { label: 'inline-object', color: 'blue' },
    include: ['tests/**/*.test.ts'],
  },
  {
    test: {
      name: { label: 'nested-object', color: 'green' },
      // Both labels deliberately own this path so generated --project args
      // remain sensitive to the static object label.
      include: ['tests/**/*.test.ts'],
    },
  },
  {
    name: { color: 'red' },
    include: ['tests/**/*.test.ts'],
  },
])
