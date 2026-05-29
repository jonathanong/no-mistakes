import { defineConfig } from '@playwright/test'
import { makeConfig } from './playwright.root-call-cycle-a-helper'

export default defineConfig({
  ...makeConfig(),
})
