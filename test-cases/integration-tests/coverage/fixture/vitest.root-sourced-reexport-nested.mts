import { defineConfig } from 'vitest/config'
import { nestedRootConfig } from './vitest.root-reexport-nested'

export default defineConfig({
  ...nestedRootConfig,
})
