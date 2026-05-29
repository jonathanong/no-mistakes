import { defineConfig } from '@playwright/test'
import { base } from './playwright.imported-nested-spread-base'

export default defineConfig({
  projects: [
    {
      ...base,
      name: 'pw-imported-nested-spread',
    },
  ],
})
