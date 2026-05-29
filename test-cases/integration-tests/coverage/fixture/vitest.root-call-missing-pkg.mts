import { defineConfig } from 'vitest/config'
import { makeConfig } from '@no-mistakes-test-nonexistent'

export default defineConfig({
  ...makeConfig(),
})
