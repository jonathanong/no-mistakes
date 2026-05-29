import { defineConfig } from 'vitest/config'
import sharedConfig from './vitest.root-default-reexport-barrel'

export default defineConfig({
  ...sharedConfig,
})
