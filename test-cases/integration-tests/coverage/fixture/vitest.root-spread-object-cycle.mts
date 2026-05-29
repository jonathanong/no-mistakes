import { defineConfig } from 'vitest/config'

// Two bindings that spread each other - covers object_seen cycle detection
const configA: any = { ...configB }
const configB: any = { ...configA }

export default defineConfig({
  ...configA,
})
