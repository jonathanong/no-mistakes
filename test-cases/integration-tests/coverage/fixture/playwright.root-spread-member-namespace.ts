import { defineConfig } from '@playwright/test'
import * as allConfigs from './playwright.root-spread-member-namespace-source'

const merged = { ...allConfigs }

export default defineConfig({
  ...merged.web,
})
