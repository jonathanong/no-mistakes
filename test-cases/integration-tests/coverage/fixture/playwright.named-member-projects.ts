import { defineConfig } from '@playwright/test'
import { groups } from './playwright.named-member-projects-helper'

export default defineConfig({
  projects: groups.web,
})
