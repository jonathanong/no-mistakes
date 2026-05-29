import { defineConfig } from 'vitest/config'
import { shared } from './vitest.root-import-then-export-barrel'

export default defineConfig({
  ...shared,
})
