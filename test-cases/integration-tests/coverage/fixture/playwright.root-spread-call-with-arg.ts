import { defineConfig } from '@playwright/test'
import { makeConfig } from './playwright.root-spread-call-with-arg-helper'

export default defineConfig({
  ...makeConfig(42),
})
