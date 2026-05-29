import { defineConfig } from '@playwright/test'
import { makeShared } from './playwright.root-call-import-helper'

export default defineConfig({
  ...makeShared(),
})
