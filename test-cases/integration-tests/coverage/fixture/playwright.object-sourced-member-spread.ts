import { defineConfig } from '@playwright/test'
import { bases } from './playwright.object-sourced-member-spread-barrel'

export default defineConfig({
  projects: [
    {
      ...bases.web,
      name: 'pw-object-sourced-member-spread',
    },
  ],
})
