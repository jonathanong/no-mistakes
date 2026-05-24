import defaultProjects, { apiProjects, webProjects } from './vitest.projects-helper'
import * as projectHelpers from './vitest.projects-helper'
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

const recursiveCall = () => recursiveCall()

const projects = [
  ...projectHelpers.sameNameProjects(),
]

export default defineConfig({
  test: {
    projects: [
      ...webProjects(),
      ...defaultProjects,
      ...apiProjects,
      ...projectHelpers.namespaceProjects(),
      ...projects,
      ...recursiveCall(),
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
