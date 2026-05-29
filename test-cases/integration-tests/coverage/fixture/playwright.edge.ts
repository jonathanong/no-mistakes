import { defineConfig } from '@playwright/test'
import defaultArrayProjects from './playwright.default-array'
import defaultArrowProjects from './playwright.default-arrow'
import defaultArrowBlockProjects from './playwright.default-arrow-block'
import defaultCallProjects from './playwright.default-call'
import defaultExportedConstProjects from './playwright.default-exported-const'
import defaultFunctionProjects from './playwright.default-function'
import defaultIdentifierArrayProjects from './playwright.default-identifier-array'
import defaultIdentifierFunctionProjects from './playwright.default-identifier-function'
import defaultIdentifierImportProjects from './playwright.default-identifier-import'
import defaultLiteralProjects from './playwright.default-literal'
import missingDefaultProjects from './playwright.default-identifier-missing'
import * as edge from './playwright.edge-source'
import {
  destructuredProjects,
  importedLocalProjects,
  localAliasProjects,
  localFunctionProjects,
  missingReexportProjects,
  namedNamespaceProjects,
  namedFunctionProjects,
  namedVarProjects,
  noMatchingDeclaration,
  reexportedProjects,
  sourcedReexportProjects,
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

export default defineConfig({
  projects: [
    ...parenthesizedProjects,
    ...(true ? [] : []),
    ...unknownProjects,
    ...recursiveIdentifierProjects,
    ...functionExpressionProjects(),
    ...emptyFunctionExpressionProjects(),
    ...blockArrowProjects(),
    ...topLevelFunctionProjects(),
    ...emptyTopLevelFunctionProjects(),
    ...returnOnlyProjects(),
    ...recursiveCallProjects(),
    ...namedVarProjects,
    ...namedFunctionProjects(),
    ...noMatchingDeclaration,
    ...destructuredProjects,
    ...importedLocalProjects,
    ...localAliasProjects,
    ...localFunctionProjects(),
    ...reexportedProjects,
    ...sourcedReexportProjects,
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
    ...defaultCallProjects,
    ...defaultExportedConstProjects,
    ...defaultFunctionProjects(),
    ...defaultIdentifierArrayProjects,
    ...defaultIdentifierFunctionProjects(),
    ...defaultIdentifierImportProjects(),
    ...defaultLiteralProjects,
    ...missingDefaultProjects,
  ],
})
