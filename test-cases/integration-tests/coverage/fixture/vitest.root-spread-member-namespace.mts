import { defineConfig } from 'vitest/config'
import * as allConfigs from './vitest.root-spread-member-namespace-source'

const merged = { ...allConfigs }

export default defineConfig({
  ...merged.web,
})
