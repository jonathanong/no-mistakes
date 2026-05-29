import { defineConfig } from '@playwright/test'

const splitConfig = {
  projects: [
    {
      name: 'root-spread',
      testMatch: ['root-spread/**/*.spec.ts'],
    },
  ],
}

const computedConfig = {
  ['projects']: [],
}

export default defineConfig({
  ...computedConfig,
  ...splitConfig,
})
