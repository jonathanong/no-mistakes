import { defineConfig } from 'vitest/config'
import shared from './vitest.root-spread-overrides-test-helper'

export default defineConfig({
  test: {
    projects: [
      {
        test: {
          name: 'root-spread-overrides-local',
          include: ['root-spread-overrides-local/**/*.test.ts'],
        },
      },
    ],
  },
  ...shared,
})
