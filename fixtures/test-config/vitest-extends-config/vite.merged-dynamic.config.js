import { defineConfig, mergeConfig } from 'vitest/config'

const shared = defineConfig({ test: { setupFiles: './must-not-trust.ts' } })
const dynamic = process.env.MERGED_CONFIG

export default mergeConfig(shared, dynamic)
