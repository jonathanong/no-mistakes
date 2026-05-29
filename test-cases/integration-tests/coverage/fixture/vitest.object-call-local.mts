import { defineConfig } from 'vitest/config'

function makeProject() {
  const include = ['vitest-object-call-local/**/*.test.ts']
  return {
    test: {
      name: 'vitest-object-call-local',
      include,
    },
  }
}

export default defineConfig({
  test: {
    projects: [makeProject()],
  },
})
