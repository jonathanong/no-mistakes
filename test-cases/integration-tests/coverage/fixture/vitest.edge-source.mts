import { reexportedProjects as importedNamedProjects } from './vitest.projects-source'
import { cycleProjects } from './vitest.edge-cycle-a'

const localAliasProjects = [
  {
    test: {
      name: 'local-alias',
      include: ['local-alias/**/*.test.ts'],
    },
  },
]

function localFunctionProjects() {
  return [
    {
      test: {
        name: 'local-function',
        include: ['local-function/**/*.test.ts'],
      },
    },
  ]
}

export const namedVarProjects = [
  {
    test: {
      name: 'named-var',
      include: ['named-var/**/*.test.ts'],
    },
  },
]

export const exportedSpecifierProjects = [
  {
    test: {
      name: 'exported-specifier',
      include: ['exported-specifier/**/*.test.ts'],
    },
  },
]

export function namedFunctionProjects() {
  return [
    {
      test: {
        name: 'named-function',
        include: ['named-function/**/*.test.ts'],
      },
    },
  ]
}

export function overloadedProjects(): unknown[]
export function overloadedProjects() {
  return [
    {
      test: {
        name: 'overloaded-function',
        include: ['overloaded-function/**/*.test.ts'],
      },
    },
  ]
}

export const namespaceProjects = [
  {
    test: {
      name: 'edge-namespace',
      include: ['edge-namespace/**/*.test.ts'],
    },
  },
]

export const namespaceCallProjects = () => [
  {
    test: {
      name: 'edge-namespace-call',
      include: ['edge-namespace-call/**/*.test.ts'],
    },
  },
]

export const namespaceTestOptions = {
  include: ['namespace-test-options-spread/**/*.test.ts'],
  exclude: ['namespace-test-options-spread/**/*.skip.ts'],
}

export const unrelatedProjects = []
export const { destructuredProjects } = {
  destructuredProjects: [
    {
      test: {
        name: 'destructured-export',
        include: ['destructured-export/**/*.test.ts'],
      },
    },
  ],
}

const computedKey = 'computedProjects'
export const { [computedKey]: computedDestructuredProjects } = {
  computedProjects: [
    {
      test: {
        name: 'computed-destructured',
        include: ['computed-destructured/**/*.test.ts'],
      },
    },
  ],
}

export { localAliasProjects, localFunctionProjects, importedNamedProjects, cycleProjects }
export { exportedSpecifierProjects as exportedSpecifierAliasProjects }
export type { typedReexportProjects } from './vitest.edge-types'
export { typedReexportProjects } from './vitest.projects-source'
export { reexportedProjects as sourcedReexportProjects } from './vitest.projects-source'
export * from './vitest.projects-source'
export { missingLocalProjects }
