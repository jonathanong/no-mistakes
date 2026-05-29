import { defineConfig } from 'vitest/config'
import { configs } from './vitest.test-named-member-spread-helper'

export default defineConfig({
  test: {
    ...configs.web,
  },
})
