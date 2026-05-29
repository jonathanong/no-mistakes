import { defineConfig } from '@playwright/test'
import { bases } from './playwright.object-member-star-barrel-re'

export default defineConfig({
  projects: [
    {
      ...bases.web,
      name: 'pw-object-member-star-barrel-fallback',
    },
  ],
})
