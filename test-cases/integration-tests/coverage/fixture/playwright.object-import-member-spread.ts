import { defineConfig } from '@playwright/test'
import { bases } from './playwright.object-import-member-spread-barrel'

export default defineConfig({
  projects: [
    {
      ...bases.web,
      name: 'pw-object-import-member-spread',
    },
  ],
})
