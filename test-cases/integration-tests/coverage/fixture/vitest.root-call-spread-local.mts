import { defineConfig } from 'vitest/config'

function makeConfig() {
  const projects = [
    {
      test: {
        name: 'vitest-root-call-local',
        include: ['vitest-root-call-local/**/*.test.ts'],
      },
    },
  ]
  return { test: { projects } }
}

export default defineConfig({ ...makeConfig() })
