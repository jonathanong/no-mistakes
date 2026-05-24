import { apiProjects, webProjects } from './vitest.projects-helper'
import { defineConfig } from 'vitest/config'

const localProjects = [
  {
    test: {
      name: 'local',
      include: ['local/**/*.test.ts'],
    },
  },
]

const dynamicInclude = ['dynamic/**/*.test.ts']

export default defineConfig({
  test: {
    projects: [
      ...webProjects(),
      ...apiProjects,
      ...localProjects,
      {
        test: {
          name: 'composed',
          include: dynamicInclude,
        },
      },
    ],
  },
})
