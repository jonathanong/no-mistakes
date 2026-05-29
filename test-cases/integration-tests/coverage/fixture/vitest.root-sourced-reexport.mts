import { defineConfig } from 'vitest/config'
import { sharedConfig } from './vitest.root-sourced-reexport-barrel'

export default defineConfig({
  ...sharedConfig,
})
