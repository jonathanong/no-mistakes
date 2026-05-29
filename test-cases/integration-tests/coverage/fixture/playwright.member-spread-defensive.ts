import { defineConfig } from '@playwright/test'
import * as allBases from './playwright.member-spread-namespace-source'

const localBase = [{ name: 'pw-local-base', testMatch: ['pw-local-base/**/*.spec.ts'] }]
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
  projects: mixed.web,
})
