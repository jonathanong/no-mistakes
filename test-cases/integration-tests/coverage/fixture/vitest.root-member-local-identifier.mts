import { defineConfig } from 'vitest/config'
import { webConfig } from './vitest.root-member-local-identifier-source'

const configs = { web: webConfig }

export default defineConfig({
  ...configs.web,
})
