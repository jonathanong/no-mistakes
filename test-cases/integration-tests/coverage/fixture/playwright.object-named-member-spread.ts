import { defineConfig } from '@playwright/test'
import { bases } from './playwright.object-named-member-spread-source'

export default defineConfig({
  projects: [
    {
      ...bases.web,
      name: 'pw-object-named-member-spread',
    },
  ],
})
