import { defineConfig } from '@playwright/test'
import { base } from './playwright.object-import-then-export-barrel'

export default defineConfig({
  projects: [
    {
      ...base,
      name: 'pw-object-import-then-export',
    },
  ],
})
