import { defineConfig } from 'vitest/config'

export default defineConfig({
  test: {
    projects: [
      {
        test: {
          name: 'stories',
          include: ['web/storybook/**/*.stories.tsx', 'web/storybook/**/*.test.tsx'],
          exclude: ['web/storybook/skip/**'],
        },
      },
      {
        test: {
          name: 'browser',
          include: ['web/storybook/button.stories.tsx'],
        },
      },
      {
        test: {
          name: 'all-specs',
          include: ['e2e/home.pw.ts'],
        },
      },
    ],
  },
})
