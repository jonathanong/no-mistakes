import { defineConfig } from '@playwright/test'

const bases = {
  web: {
    testMatch: ['pw-local-member-spread/**/*.spec.ts'],
    testIgnore: ['pw-local-member-spread/**/*.skip.ts'],
  },
}

export default defineConfig({
  projects: [
    {
      ...bases.web,
      name: 'pw-local-member-spread',
    },
  ],
})
