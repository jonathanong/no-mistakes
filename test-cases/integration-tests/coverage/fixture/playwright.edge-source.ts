import { reexportedProjects as importedLocalProjects } from './playwright.edge-reexport-source'

export const namedVarProjects = [
  {
    name: 'pw-named-var',
    testMatch: ['pw-named-var/**/*.spec.ts'],
  },
]

export function namedFunctionProjects() {
  return [
    {
      name: 'pw-named-function',
      testMatch: ['pw-named-function/**/*.spec.ts'],
    },
  ]
}

const localAliasProjects = [
  {
    name: 'pw-local-alias',
    testMatch: ['pw-local-alias/**/*.spec.ts'],
  },
]

function localFunctionProjects() {
  return [
    {
      name: 'pw-local-function',
      testMatch: ['pw-local-function/**/*.spec.ts'],
    },
  ]
}

export const namespaceProjects = [
  {
    name: 'pw-namespace',
    testMatch: ['pw-namespace/**/*.spec.ts'],
  },
]

export const namespaceCallProjects = () => [
  {
    name: 'pw-namespace-call',
    testMatch: ['pw-namespace-call/**/*.spec.ts'],
  },
]

export const namedNamespaceProjects = {}
export const { destructuredProjects } = {
  destructuredProjects: [
    {
      name: 'pw-destructured',
      testMatch: ['pw-destructured/**/*.spec.ts'],
    },
  ],
}

export const { projects: aliasedDestructuredProjects } = {
  projects: [
    {
      name: 'pw-aliased-destructured',
      testMatch: ['pw-aliased-destructured/**/*.spec.ts'],
    },
  ],
}

const computedKey = 'computedProjects'
export const { [computedKey]: computedDestructuredProjects } = {
  computedProjects: [
    {
      name: 'pw-computed-destructured',
      testMatch: ['pw-computed-destructured/**/*.spec.ts'],
    },
  ],
}

export const { projects: { nestedDestructuredProjects } } = {
  projects: {
    nestedDestructuredProjects: [
      {
        name: 'pw-nested-destructured',
        testMatch: ['pw-nested-destructured/**/*.spec.ts'],
      },
    ],
  },
}

export const { missingValueProjects } = {}
export const { nonObjectInitProjects } = []
export const [arrayBindingProjects] = [
  [
    {
      name: 'pw-array-binding',
      testMatch: ['pw-array-binding/**/*.spec.ts'],
    },
  ],
]

const identifierElementProject = {
  name: 'pw-identifier-element',
  testMatch: ['pw-identifier-element/**/*.spec.ts'],
}

export const identifierElementProjects = [identifierElementProject]

export { localAliasProjects, localFunctionProjects }
export { importedLocalProjects }
export { noMatchingDeclaration }
export { reexportedProjects as sourcedReexportProjects } from './playwright.edge-reexport-source'
export { starPrecedenceProjects } from './playwright.star-explicit'
export { type typeOnlyProjects as specifierTypeProjects } from './playwright.edge-types'
export type { typeOnlyProjects } from './playwright.edge-types'
export { typeOnlyProjects } from './playwright.edge-type-runtime'
export { missingReexportProjects } from './playwright.edge-empty-reexport-source'
export * from './playwright.edge-reexport-source'
export * from './playwright.edge-empty-reexport-source'
