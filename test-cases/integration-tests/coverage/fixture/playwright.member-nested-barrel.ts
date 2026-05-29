import { defineConfig } from '@playwright/test'
import { groups } from './playwright.member-nested'

export default defineConfig({
  projects: groups.web,
})
