import { defineConfig } from 'vitest/config'
import { configs } from './vitest.root-member-import-then-export-barrel'

export default defineConfig({
  ...configs.web,
})
