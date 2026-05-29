import { defineConfig } from 'vitest/config'
import { sharedRootConfig } from './vitest.root-imported-config-helper'

export default defineConfig({
  ...sharedRootConfig,
})
