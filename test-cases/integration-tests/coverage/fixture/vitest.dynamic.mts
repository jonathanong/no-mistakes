import defaultProjects, {
  aliasDefaultProjects,
  apiProjects,
  reexportedProjects,
  toolingProjects,
  webProjects,
} from './vitest.projects-helper'
import defaultArrowProjects from './vitest.projects-default-arrow'
import defaultCallArgProjects from './vitest.projects-default-call-arg'
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

export const localExportedProjects = [
  {
    test: {
      name: 'local-exported',
      include: ['local-exported/**/*.test.ts'],
    },
  },
]

export function localExportedFunctionProjects() {
  return [
    {
      test: {
        name: 'local-exported-function',
        include: ['local-exported-function/**/*.test.ts'],
      },
    },
  ]
}

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
      ...defaultCallArgProjects,
      ...defaultCallProjects,
      ...defaultFunctionProjects(),
      ...edgeProjects,
      ...aliasDefaultProjects(),
      ...apiProjects,
      ...reexportedProjects,
      ...projectHelpers.namespaceProjects(),
      ...projectHelpers.namespaceArrayProjects,
      ...toolingProjects,
      ...projects,
      ...localExportedProjects,
      ...localExportedFunctionProjects(),
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
