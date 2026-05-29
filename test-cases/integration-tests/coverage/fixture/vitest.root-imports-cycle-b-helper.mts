import { helperA } from './vitest.root-imports-cycle-a-helper'

export const helperB = {
  ...helperA,
  test: {
    projects: [
      {
        test: {
          name: 'vitest-root-imports-cycle',
          include: ['vitest-root-imports-cycle/**/*.test.ts'],
        },
      },
    ],
  },
}
