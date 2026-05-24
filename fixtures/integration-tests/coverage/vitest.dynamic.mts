import { webProjects } from './vitest.projects-helper'
import { defineConfig } from 'vitest/config'

const localProjects = [
  {
    test: {
      name: 'local',
      include: ['local/**/*.test.ts'],
    },
  },
]

export default defineConfig({
  test: {
    projects: [
      ...webProjects(),
      ...localProjects,
    ],
  },
})
