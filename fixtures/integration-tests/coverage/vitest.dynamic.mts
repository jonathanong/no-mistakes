import defaultProjects, {
  aliasDefaultProjects,
  apiProjects,
  reexportedProjects,
  webProjects,
} from './vitest.projects-helper'
import defaultCallProjects from './vitest.projects-default-call'
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
      ...defaultCallProjects,
      ...aliasDefaultProjects(),
      ...apiProjects,
      ...reexportedProjects,
      ...projectHelpers.namespaceProjects(),
      ...projectHelpers.namespaceArrayProjects,
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
