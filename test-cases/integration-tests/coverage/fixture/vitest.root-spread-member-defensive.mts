import { defineConfig } from 'vitest/config'

const localBase = {}
const wrappers = { inner: {} }

// named property triggers the non-spread continue
// ...wrappers.inner is a MemberExpression spread, not Identifier
// ...localBase is a local-only binding not in imports
const mixed = {
  named: 1,
  ...wrappers.inner,
  ...localBase,
}

export default defineConfig({
  ...mixed.web,
  test: {
    projects: [{ test: { name: 'vitest-root-spread-member-defensive', include: ['vitest-root-spread-member-defensive/**/*.test.ts'] } }],
  },
})
