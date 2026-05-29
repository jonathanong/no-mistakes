import { defineConfig } from '@playwright/test'

const sharedOptions = {
  projects: [
    {
      name: 'root-define-config-spread',
      testMatch: ['root-define-config-spread/**/*.spec.ts'],
    },
  ],
}

const parenthesizedOptions = ({
  projects: [
    {
      name: 'root-define-config-parenthesized-spread',
      testMatch: ['root-define-config-parenthesized-spread/**/*.spec.ts'],
    },
  ],
})

const shared = defineConfig(sharedOptions)
const parenthesized = defineConfig((parenthesizedOptions))
const ignored = defineConfig(true)

export default defineConfig({
  ...ignored,
  ...parenthesized,
  ...shared,
})
