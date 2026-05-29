import { defineConfig } from 'vitest/config'

// test: is set to a literal (not an object or spread) - covers found = None branch
const testValue = 42

export default defineConfig({
  test: testValue,
})
