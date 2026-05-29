import { defineConfig } from '@playwright/test'
import * as allBases from './playwright.member-spread-namespace-source'

const merged = { ...allBases }

export default defineConfig({
  projects: merged.web,
})
