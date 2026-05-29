import { defineConfig } from '@playwright/test'
import { configs } from '@missing-no-mistakes-pkg'

export default defineConfig({
  ...configs.web,
  projects: [{ name: 'pw-root-member-missing-import', testMatch: ['pw-root-member-missing-import/**/*.spec.ts'] }],
})
