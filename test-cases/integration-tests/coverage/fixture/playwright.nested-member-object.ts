import { defineConfig } from '@playwright/test'

const wrappers = {
  inner: { web: [{ name: 'pw-nested-member-object', testMatch: ['**/*.spec.ts'] }] },
}

// `wrappers.inner.web` is a nested static member, so namespace_member_options
// sees a non-identifier object and returns no projects instead of guessing.
export default defineConfig({
  projects: wrappers.inner.web,
})
