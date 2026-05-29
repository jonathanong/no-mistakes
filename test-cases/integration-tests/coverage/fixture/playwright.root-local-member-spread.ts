import { defineConfig } from '@playwright/test'

const configs = {
  web: { projects: [{ name: 'pw-root-local-member-spread', testMatch: 'pw-root-local-member-spread/**/*.spec.ts' }] },
}

export default defineConfig({
  ...configs.web,
})
