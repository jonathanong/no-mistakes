import { defineConfig } from 'vitest/config'
import defaultConfigs from './vitest.root-member-default-import-source'

export default defineConfig({
  ...defaultConfigs.web,
})
