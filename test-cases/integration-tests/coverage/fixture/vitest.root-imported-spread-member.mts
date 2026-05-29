import { defineConfig } from 'vitest/config'
import { shared } from './vitest.root-imported-spread-member-source'

const configs = { ...shared }

export default defineConfig({
  ...configs.web,
})
