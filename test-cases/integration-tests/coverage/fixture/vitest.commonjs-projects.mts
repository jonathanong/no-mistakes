import projects from './vitest.projects-commonjs.cjs'
import { defineConfig } from 'vitest/config'

export default defineConfig({
  test: {
    projects,
  },
})
