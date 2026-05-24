import importedDefaultProjects from './vitest.projects-source'

export const webProjects = () => [
  {
    test: {
      name: 'web',
      include: ['web/**/*.test.ts'],
      exclude: ['web/**/*.skip.ts'],
    },
  },
]

export const apiProjects = [
  {
    test: {
      name: 'api',
      include: ['api/**/*.test.ts'],
    },
  },
]

export const namespaceProjects = () => [
  {
    test: {
      name: 'namespace',
      include: ['namespace/**/*.test.ts'],
    },
  },
]

export const namespaceArrayProjects = [
  {
    test: {
      name: 'namespace-array',
      include: ['namespace-array/**/*.test.ts'],
    },
  },
]

const defaultProjects = [
  {
    test: {
      name: 'default-import',
      include: ['default-import/**/*.test.ts'],
    },
  },
]

const projects = [
  {
    test: {
      name: 'same-name-import',
      include: ['same-name-import/**/*.test.ts'],
    },
  },
]

export const sameNameProjects = () => projects
export { defaultProjects as default }
export { reexportedProjects } from './vitest.projects-source'
export const aliasDefaultProjects = () => importedDefaultProjects
