import defaultProjects, {
  aliasDefaultProjects,
  apiProjects,
  reexportedProjects,
  webProjects,
} from './vitest.projects-helper'
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

const recursiveCall = () => recursiveCall()

const projects = [
  ...projectHelpers.sameNameProjects(),
]

export default defineConfig({
  test: {
    projects: [
      ...webProjects(),
      ...defaultProjects,
      ...aliasDefaultProjects(),
      ...apiProjects,
      ...reexportedProjects,
      ...projectHelpers.namespaceProjects(),
      ...projects,
      ...recursiveCall(),
      ...localProjects,
      {
        test: {
          name: 'composed',
          include: ['dynamic/**/*.test.ts'],
        },
      },
    ],
  },
})
