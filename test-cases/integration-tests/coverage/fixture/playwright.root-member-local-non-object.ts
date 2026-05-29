import { defineConfig } from '@playwright/test'

const configs = {
  web: 'not-an-object',
}

export default defineConfig({
  ...configs.web,
  projects: [{ name: 'pw-root-member-local-non-object', testMatch: ['pw-root-member-local-non-object/**/*.spec.ts'] }],
})
