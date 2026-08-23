import * as vitest from 'vitest/config'

export default vitest.defineWorkspace([
  { test: { name: 'namespace-project', include: ['namespace/**/*.test.ts'] } },
])
