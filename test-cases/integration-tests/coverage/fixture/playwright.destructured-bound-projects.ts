import { defineConfig } from '@playwright/test'
import { projects } from './playwright.destructured-bound-projects-helper'

export default defineConfig({
  projects,
})
