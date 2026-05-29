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

export { localAliasProjects, localFunctionProjects }
export { importedLocalProjects }
export { noMatchingDeclaration }
export { reexportedProjects as sourcedReexportProjects } from './playwright.edge-reexport-source'
export { missingReexportProjects } from './playwright.edge-empty-reexport-source'
export * from './playwright.edge-reexport-source'
export * from './playwright.edge-empty-reexport-source'
