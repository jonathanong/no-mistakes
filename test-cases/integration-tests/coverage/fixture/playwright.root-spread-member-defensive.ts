import { defineConfig } from '@playwright/test'

const localBase = {}
const wrappers = { inner: {} }

// named property triggers the non-spread continue (line 110)
// ...wrappers.inner is a MemberExpression spread, not Identifier (line 113 continue)
// ...localBase is a local-only binding not in imports (line 116 continue)
const mixed = {
  named: 1,
  ...wrappers.inner,
  ...localBase,
}

export default defineConfig({
  ...mixed.web,
  projects: [{ name: 'pw-root-spread-member-defensive', testMatch: ['pw-root-spread-member-defensive/**/*.spec.ts'] }],
})
