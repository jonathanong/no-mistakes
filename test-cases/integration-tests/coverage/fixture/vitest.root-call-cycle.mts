import { defineConfig } from 'vitest/config'
import { makeConfig } from './vitest.root-call-cycle-a-helper'

export default defineConfig({
  ...makeConfig(),
})
