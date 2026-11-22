import { configDefaults as defaults, defineConfig } from 'vitest/config'

export default defineConfig((defaults) => ({ test: { exclude: [...defaults.exclude] } }))
