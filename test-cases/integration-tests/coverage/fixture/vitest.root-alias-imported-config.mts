import { defineConfig } from 'vitest/config'
import { sharedAliasRootConfig } from './vitest.root-imported-config-helper'

export default defineConfig({
  ...sharedAliasRootConfig,
})
