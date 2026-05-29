import { defineConfig } from 'vitest/config'
import { helperA } from './vitest.root-imports-cycle-a-helper'

export default defineConfig({
  ...helperA,
})
