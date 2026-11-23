import { configDefaults as defaults, defineConfig } from 'vitest/config'
import { base, overrides } from './dynamic'

export function createBareFactory() {
  return { test: { exclude: ['custom/**'] } }
}

export function createWrappedFactory() {
  return defineConfig({
    ...base,
    test: {
      ...base.test,
      exclude: ['**/node_modules/**', 'custom/**'],
    },
  })
}

export function createSafeFactory() {
  return {
    test: {
      exclude: [...defaults.exclude, 'custom/**'],
      coverage: { exclude: ['custom/**'] },
    },
    exclude: ['custom/**'],
  }
}

export function createOverriddenFactory() {
  return { test: { exclude: ['custom/**'], ...overrides } }
}
