import { defineConfig } from '@playwright/test'
import { groups } from './playwright.member-namespace-star-barrel'

export default defineConfig({
  projects: groups.web,
})
