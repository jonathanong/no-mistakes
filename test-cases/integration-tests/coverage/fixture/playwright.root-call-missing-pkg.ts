import { defineConfig } from '@playwright/test'
import { makeConfig } from '@no-mistakes-test-nonexistent'

export default defineConfig({
  ...makeConfig(),
})
