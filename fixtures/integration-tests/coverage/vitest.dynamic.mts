import defaultProjects, {
  aliasDefaultProjects,
  apiProjects,
  reexportedProjects,
  webProjects,
} from './vitest.projects-helper'
import defaultArrowProjects from './vitest.projects-default-arrow'
import defaultCallProjects from './vitest.projects-default-call'
import defaultFunctionProjects from './vitest.projects-default-function'
import edgeProjects from './vitest.projects-edge-default-array'
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
      ...defaultArrowProjects(),
      ...defaultCallProjects,
      ...defaultFunctionProjects(),
      ...edgeProjects,
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
