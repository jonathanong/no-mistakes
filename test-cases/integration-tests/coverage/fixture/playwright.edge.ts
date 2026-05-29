import { defineConfig } from '@playwright/test'
import { ambiguousStarProjects } from './playwright.ambiguous-star-barrel'
import defaultArrayProjects from './playwright.default-array'
import defaultArrowProjects from './playwright.default-arrow'
import defaultArrowBlockProjects from './playwright.default-arrow-block'
import defaultAsProjects from './playwright.default-as-array'
import defaultCallArgProjects from './playwright.default-call-arg'
import defaultCallProjects from './playwright.default-call'
import defaultCommonjsProjects from './playwright.projects-commonjs.cjs'
import defaultDirectAsProjects from './playwright.default-direct-as-array'
import defaultDirectSatisfiesProjects from './playwright.default-direct-satisfies-array'
import defaultDirectTypeAssertionProjects from './playwright.default-direct-type-assertion-array'
import defaultExportedConstProjects from './playwright.default-exported-const'
import defaultFunctionProjects from './playwright.default-function'
import defaultIdentifierArrayProjects from './playwright.default-identifier-array'
import defaultIdentifierFunctionProjects from './playwright.default-identifier-function'
import defaultIdentifierImportProjects from './playwright.default-identifier-import'
import defaultLiteralProjects from './playwright.default-literal'
import defaultNonNullProjects from './playwright.default-non-null-array'
import defaultObjectProject from './playwright.default-object'
import { importedConstantProject } from './playwright.imported-object-constant-base'
import nonSpreadImportedArrayProjects from './playwright.non-spread-imported-array'
import defaultSatisfiesProjects from './playwright.default-satisfies-array'
import defaultTypeAssertionProjects from './playwright.default-type-assertion-array'
import defaultWrappedArrayProjects from './playwright.default-wrapped-array'
import missingDefaultProjects from './playwright.default-identifier-missing'
import { nonambiguousStarProjects } from './playwright.nonambiguous-star-barrel'
import { namespaceStarProjects } from './playwright.namespace-star-barrel'
import { shadowedRuntimeProjects } from './playwright.edge-reexport-source'
import { typeStarProjects } from './playwright.type-star-barrel'
import { type specifierTypeOnlyProjects } from './playwright.edge-reexport-source'
import type { shadowedRuntimeProjects } from './playwright.edge-types'
import * as edge from './playwright.edge-source'
import {
  aliasedDestructuredProjects,
  arrayBindingProjects,
  computedDestructuredProjects,
  destructuredProjects,
  identifierElementProjects,
  importedLocalProjects,
  localAliasProjects,
  localFunctionProjects,
  missingReexportProjects,
  missingValueProjects,
  namedNamespaceProjects,
  namedFunctionProjects,
  namedVarProjects,
  nestedDestructuredProjects,
  nonObjectInitProjects,
  noMatchingDeclaration,
  reexportedProjects,
  sourcedReexportProjects,
  starPrecedenceProjects,
  specifierTypeProjects,
  typeOnlyProjects,
} from './playwright.edge-source'
import { cycleProjects } from './playwright.cycle-a'
import { missingFileProjects } from './playwright.missing-file'
import { unreadableProjects } from './playwright.unreadable'

const parenthesizedProjects = ([
  {
    name: 'pw-parenthesized',
    testMatch: ['pw-parenthesized/**/*.spec.ts'],
  },
])

const functionExpressionProjects = function () {
  return [
    {
      name: 'pw-function-expression',
      testMatch: ['pw-function-expression/**/*.spec.ts'],
    },
  ]
}

const emptyFunctionExpressionProjects = function () {}

const blockArrowProjects = () => {
  return [
    {
      name: 'pw-block-arrow',
      testMatch: ['pw-block-arrow/**/*.spec.ts'],
    },
  ]
}

function topLevelFunctionProjects() {
  return [
    {
      name: 'pw-top-level-function',
      testMatch: ['pw-top-level-function/**/*.spec.ts'],
    },
  ]
}

function emptyTopLevelFunctionProjects() {
  const ignored = []
  return ignored
}

function returnOnlyProjects() {
  return
}

const recursiveIdentifierProjects = recursiveIdentifierProjects
const recursiveCallProjects = () => recursiveCallProjects()
const wrappedHelperProjects = (() => [
  {
    name: 'pw-wrapped-helper',
    testMatch: ['pw-wrapped-helper/**/*.spec.ts'],
  },
]) satisfies () => unknown[]
const localProjectGroups = {
  web: [
    {
      name: 'pw-local-member-array',
      testMatch: ['pw-local-member-array/**/*.spec.ts'],
    },
  ],
}

function nonSpreadCallArrayProjects() {
  return [
    {
      name: 'pw-non-spread-call-array',
      testMatch: ['pw-non-spread-call-array/**/*.spec.ts'],
    },
  ]
}

function objectProject() {
  return {
    name: 'pw-object-call-project',
    testMatch: ['pw-object-call-project/**/*.spec.ts'],
  }
}

const objectArrowProject = () => ({
  name: 'pw-object-call-arrow-project',
  testMatch: ['pw-object-call-arrow-project/**/*.spec.ts'],
})

const objectBlockProject = () => {
  return {
    name: 'pw-object-call-block-project',
    testMatch: ['pw-object-call-block-project/**/*.spec.ts'],
  }
}

const objectFunctionProject = function () {
  return {
    name: 'pw-object-call-function-project',
    testMatch: ['pw-object-call-function-project/**/*.spec.ts'],
  }
}

const objectExpressionProject = {
  name: 'pw-object-call-expression-project',
  testMatch: ['pw-object-call-expression-project/**/*.spec.ts'],
}

const recursiveObjectProject = () => recursiveObjectProject()

function objectNoReturnProject() {
  const ignored = true
}

function objectReturnOnlyProject() {
  return
}

export default defineConfig({
  projects: [
    ,
    ...parenthesizedProjects,
    ...(true ? [] : []),
    ...unknownProjects,
    ...recursiveIdentifierProjects,
    ...functionExpressionProjects(),
    ...emptyFunctionExpressionProjects(),
    ...blockArrowProjects(),
    ...topLevelFunctionProjects('ignored'),
    ...topLevelFunctionProjects(),
    ...emptyTopLevelFunctionProjects(),
    ...returnOnlyProjects(),
    ...recursiveCallProjects(),
    ...wrappedHelperProjects(),
    ...localProjectGroups.web,
    ...localProjectGroups.missing,
    [
      {
        name: 'pw-direct-nested-array-should-not-flatten',
        testMatch: ['pw-direct-nested-array-should-not-flatten/**/*.spec.ts'],
      },
    ],
    nonSpreadCallArrayProjects(),
    nonSpreadImportedArrayProjects,
    ({}).objectProject(),
    missingObjectProject(),
    recursiveObjectProject(),
    objectProject(),
    objectArrowProject(),
    objectBlockProject(),
    objectFunctionProject(),
    objectExpressionProject(),
    objectNoReturnProject(),
    objectReturnOnlyProject(),
    {
      ...importedConstantProject,
      name: 'pw-imported-constant-spread',
    },
    ...ambiguousStarProjects,
    ...nonambiguousStarProjects,
    ...namedVarProjects,
    ...namedFunctionProjects(),
    ...noMatchingDeclaration,
    ...destructuredProjects,
    ...aliasedDestructuredProjects,
    ...computedDestructuredProjects,
    ...nestedDestructuredProjects,
    ...missingValueProjects,
    ...nonObjectInitProjects,
    ...arrayBindingProjects,
    ...identifierElementProjects,
    ...importedLocalProjects,
    ...localAliasProjects,
    ...localFunctionProjects(),
    ...reexportedProjects,
    ...sourcedReexportProjects,
    ...starPrecedenceProjects,
    ...typeStarProjects,
    ...specifierTypeProjects,
    ...typeOnlyProjects,
    ...shadowedRuntimeProjects,
    ...missingReexportProjects,
    ...edge.namespaceProjects,
    ...edge.namespaceCallProjects(),
    ...edge.missingNamespaceProjects,
    ...edge.missingNamespaceProjects(),
    ...(true ? edge.namespaceCallProjects : edge.namespaceCallProjects)(),
    ...namedNamespaceProjects.missing,
    ...namedNamespaceProjects.missing(),
    ...unknownNamespace.namespaceProjects,
    ...unknownNamespace.namespaceProjects(),
    ...({}).namespaceProjects,
    ...({}).namespaceProjects(),
    ...cycleProjects,
    ...missingFileProjects,
    ...unreadableProjects,
    ...defaultArrayProjects,
    ...defaultArrowProjects(),
    ...defaultArrowBlockProjects(),
    ...defaultAsProjects,
    ...defaultCallArgProjects,
    ...defaultCallProjects,
    ...defaultCommonjsProjects,
    ...defaultDirectAsProjects,
    ...defaultDirectSatisfiesProjects,
    ...defaultDirectTypeAssertionProjects,
    ...defaultExportedConstProjects,
    ...defaultFunctionProjects(),
    ...defaultIdentifierArrayProjects,
    ...defaultIdentifierFunctionProjects(),
    ...defaultIdentifierImportProjects(),
    ...defaultLiteralProjects,
    ...defaultNonNullProjects,
    ...defaultWrappedArrayProjects,
    ...defaultSatisfiesProjects,
    ...defaultTypeAssertionProjects,
    defaultObjectProject,
    ...missingDefaultProjects,
    ...namespaceStarProjects,
  ],
})
