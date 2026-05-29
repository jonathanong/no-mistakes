import { defineConfig } from '@playwright/test'
import * as bases from './playwright.object-namespace-member-spread-source'

export default defineConfig({
  projects: [
    {
      ...bases.web,
      name: 'pw-object-namespace-member-spread',
    },
  ],
})
