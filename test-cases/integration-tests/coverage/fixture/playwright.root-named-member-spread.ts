import { defineConfig } from '@playwright/test'
import { configs } from './playwright.root-named-member-spread-helper'

export default defineConfig({
  ...configs.web,
})
