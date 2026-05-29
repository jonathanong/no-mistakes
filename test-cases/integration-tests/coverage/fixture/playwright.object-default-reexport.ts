import { defineConfig } from '@playwright/test'
import base from './playwright.object-default-reexport-barrel'

export default defineConfig({
  projects: [
    {
      ...base,
      name: 'pw-object-default-reexport',
    },
  ],
})
