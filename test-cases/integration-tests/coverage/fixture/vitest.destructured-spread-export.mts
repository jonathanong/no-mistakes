import { defineConfig } from 'vitest/config'
import { web } from './vitest.destructured-spread-export-helper'

export default defineConfig({
  test: { projects: web },
})
