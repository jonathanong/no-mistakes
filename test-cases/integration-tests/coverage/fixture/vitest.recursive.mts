import { defineConfig } from 'vitest/config'

const first = second
const second = first

export default defineConfig({
  test: {
    projects: [
      ...first,
    ],
  },
})
