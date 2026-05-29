import { defineConfig } from '@playwright/test'
import groups from './playwright.member-default-reexport-barrel'

export default defineConfig({
  projects: groups.web,
})
