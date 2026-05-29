import {
  importedBaseProject,
  localAliasBaseProject,
  importedTrailingBaseProject,
} from './playwright.spread-base'
import * as spreadNamespace from './playwright.spread-base'
import { reexportedBaseProject } from './playwright.spread-barrel'
import { reexportedBaseProject as starReexportedBaseProject } from './playwright.spread-star-barrel'
import { typeStarBaseProject } from './playwright.spread-type-star-barrel'
import { ambiguousBaseProject } from './playwright.spread-ambiguous-star-barrel'
import { nestedReexportedBaseProject } from './playwright.spread-nested'
import defaultSpreadBaseProject from './playwright.default-spread-base'
import { selfImportedBaseProject as importedSelfBaseProject } from './playwright.projects-helper'
import { noResolveBaseProject } from 'missing-package'
import {
  destructuredProjects as destructuredBaseProject,
  noMatchingDeclaration as noMatchingBaseProject,
} from './playwright.edge-source'
import { missingBaseProject } from './playwright.missing-file'
import { unreadableBaseProject } from './playwright.unreadable'
import defaultObjectBaseProject from './playwright.default-spread-base'
import type { importedBaseProject as typeOnlyBaseProject } from './playwright.spread-base'

const sharedProject = {
  testDir: './imported',
  testMatch: ['**/*.imported.spec.ts'],
  testIgnore: ['**/*.skip.ts'],
}

const nestedImportedSpreadProject = {
  ...importedBaseProject,
}

const recursiveBaseProject = {
  ...recursiveBaseProject,
}

const parenthesizedBaseProject = ({
  testDir: './parenthesized-spread',
  testMatch: ['**/*.parenthesized-spread.spec.ts'],
})

const constantSpreadMatch = ['**/*.constant-spread.spec.ts']
const recursiveSpreadName = recursiveSpreadName
const constantSpreadBaseProject = {
  name: recursiveSpreadName,
  testDir: './constant-spread',
  testMatch: constantSpreadMatch,
}

const nestedArrayProjects = [
  {
    name: 'nested-array-should-not-flatten',
    testMatch: ['nested-array-should-not-flatten/**/*.spec.ts'],
  },
]

function makeIgnoredConfig(_config) {
  return {}
}

export const importedPlaywrightProjects = [
  {
    ...sharedProject,
    name: 'imported',
  },
]

export const importedSpreadProjects = [
  nestedArrayProjects,
  {
    ...importedBaseProject,
    name: 'imported-spread',
  },
  {
    ...nestedImportedSpreadProject,
    name: 'nested-imported-local-spread',
  },
  {
    ...defaultSpreadBaseProject,
    name: 'default-imported-spread',
  },
  {
    ...reexportedBaseProject,
    name: 'reexported-spread',
  },
  {
    ...starReexportedBaseProject,
    name: 'star-reexported-spread',
  },
  {
    ...typeStarBaseProject,
    name: 'type-star-spread',
  },
  {
    ...constantSpreadBaseProject,
    name: 'constant-spread',
  },
  {
    ...ambiguousBaseProject,
    name: 'ambiguous-object-spread',
  },
  {
    ...nestedReexportedBaseProject,
    name: 'nested-reexported-spread',
  },
  {
    ...spreadNamespace.namespaceBaseProject,
    name: 'namespace-spread',
  },
  {
    ...localAliasBaseProject,
    name: 'local-alias-spread',
  },
  {
    ...makeIgnoredConfig({ testMatch: ['call-spread-ignored/**/*.spec.ts'] }),
    name: 'call-spread-ignored',
  },
  {
    name: 'local-before-spread',
    testDir: './local-before-spread',
    testMatch: ['**/*.local-before-spread.spec.ts'],
    testIgnore: ['**/*.local-before-spread.skip.ts'],
    ...importedTrailingBaseProject,
  },
  {
    ...recursiveBaseProject,
    ...missingLocalBaseProject,
    ...parenthesizedBaseProject,
    ...({}).missing,
    ...defaultObjectBaseProject.missing,
    ...typeOnlyBaseProject,
    ...({
      testDir: './inline-spread',
      testMatch: ['**/*.inline-spread.spec.ts'],
    }),
    ...true,
    ...noResolveBaseProject,
    ...missingBaseProject,
    ...unreadableBaseProject,
    ...noMatchingBaseProject,
    ...destructuredBaseProject,
    name: 'defensive-spreads',
    testMatch: ['defensive-spreads/**/*.spec.ts'],
  },
]

export const selfImportedBaseProject = {
  testMatch: ['**/*.self-imported-base.spec.ts'],
}

export const selfImportedSpreadProjects = [
  {
    ...importedSelfBaseProject,
    name: 'self-imported-spread',
    testMatch: ['self-imported-spread/**/*.spec.ts'],
  },
]

export const wrappedPlaywrightProjects = ([
  {
    name: 'wrapped',
    testMatch: ['wrapped/**/*.spec.ts'],
  },
] as const)

export function factoryPlaywrightProjects() {
  return [
    {
      name: 'factory',
      testDir: './factory',
      testMatch: ['**/*.factory.spec.ts'],
    },
  ]
}
