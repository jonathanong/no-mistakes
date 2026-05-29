import { defineConfig } from 'vitest/config'
import { configs } from './vitest.root-named-member-spread-helper'

export default defineConfig({
  ...configs.web,
})
