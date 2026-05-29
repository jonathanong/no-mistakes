import { defineConfig } from 'vitest/config'
import * as allBases from './vitest.member-spread-namespace-source'

const localBase = [{ test: { name: 'vitest-local-base', include: ['vitest-local-base/**/*.test.ts'] } }]
const wrappers = { inner: localBase }

// named property triggers non-spread continue
// ...wrappers.inner is MemberExpression spread (not Identifier) -> line 58 continue
// ...localBase is a local-only binding (not an import) -> line 61 continue
// ...allBases is a namespace import -> exercises namespace spread path
const mixed = {
  named: [],
  ...wrappers.inner,
  ...localBase,
  ...allBases,
}

export default defineConfig({
  test: {
    projects: mixed.web,
  },
})
