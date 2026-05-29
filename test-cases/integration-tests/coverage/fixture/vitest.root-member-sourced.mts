import { defineConfig } from 'vitest/config'
import { configs } from './vitest.root-member-sourced-barrel'

export default defineConfig({
  ...configs.web,
})
