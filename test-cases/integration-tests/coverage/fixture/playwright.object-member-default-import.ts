import { defineConfig } from '@playwright/test'
import defaultBases from './playwright.object-member-default-import-source'

export default defineConfig({
  projects: [
    {
      ...defaultBases.web,
      name: 'pw-object-member-default-import',
    },
  ],
})
