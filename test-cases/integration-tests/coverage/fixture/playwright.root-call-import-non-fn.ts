import { defineConfig } from '@playwright/test'
import { makeConfig } from './playwright.root-call-import-non-fn-helper'

export default defineConfig({
  ...makeConfig(),
})
