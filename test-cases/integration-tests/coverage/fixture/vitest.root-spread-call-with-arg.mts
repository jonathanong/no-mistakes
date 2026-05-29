import { defineConfig } from 'vitest/config'
import { makeConfig } from './vitest.root-spread-call-with-arg-helper'

export default defineConfig({
  ...makeConfig(42),
})
