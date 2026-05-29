import { defineConfig } from 'vitest/config'

// Local function with destructuring in body covers function_body_bindings continue path
function makeProject() {
  const { dir } = { dir: 'vitest-destructure-body' }
  return {
    test: {
      name: 'vitest-object-call-destructure-body',
      include: ['vitest-object-call-destructure-body/**/*.test.ts'],
    },
  }
}

export default defineConfig({
  test: {
    projects: [makeProject()],
  },
})
