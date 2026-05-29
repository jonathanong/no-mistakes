import { defineConfig } from 'vitest/config'
import { shared } from './vitest.imported-spread-member-map-source'

const configs = { ...shared }

export default defineConfig({
  test: { projects: configs.web },
})
