import { defineConfig } from 'vitest/config'
import shared from './vitest.root-spread-order-helper'

export default defineConfig({
  test: {
    projects: [
      {
        test: {
          name: 'root-spread-order-local',
          include: ['local/**/*.test.ts'],
        },
      },
    ],
    ...shared,
  },
})
