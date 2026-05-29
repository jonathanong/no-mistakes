import { defineConfig } from 'vitest/config'

const bases = {
  web: {
    test: {
      name: 'vitest-local-member-spread',
      include: ['vitest-local-member-spread/**/*.test.ts'],
      exclude: ['vitest-local-member-spread/**/*.skip.ts'],
    },
  },
}

export default defineConfig({
  test: {
    projects: [
      {
        ...bases.web,
      },
    ],
  },
})
