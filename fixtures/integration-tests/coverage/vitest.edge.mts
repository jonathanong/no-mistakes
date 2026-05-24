import defaultArrowBlockProjects from './vitest.edge-default-arrow-block'
import defaultIdentifierFunctionProjects from './vitest.edge-default-identifier-function'
import defaultIdentifierImportedProjects from './vitest.edge-default-identifier-import'
import literalDefaultProjects from './vitest.edge-default-literal'
import missingDefaultProjects from './vitest.edge-default-missing'
import * as edge from './vitest.edge-source'
import defaultObjectProjects from './vitest.projects-source'
import {
  cycleProjects,
  destructuredProjects,
  importedNamedProjects,
  localAliasProjects,
  localFunctionProjects,
  missingLocalProjects,
  namedFunctionProjects,
  namedVarProjects,
  sourcedReexportProjects,
  noMatchingDeclaration,
} from './vitest.edge-source'
import { badProjects } from './vitest.edge-bad'
import { missingFileProjects } from './vitest.missing-file'
import { packageProjects } from 'missing-package'
import { defineConfig } from 'vitest/config'

const parenthesizedProjects = ([
  {
    test: {
      name: 'parenthesized',
      include: ['parenthesized/**/*.test.ts'],
    },
  },
])

const functionExpressionProjects = function () {
  return [
    {
      test: {
        name: 'function-expression',
        include: ['function-expression/**/*.test.ts'],
      },
    },
  ]
}

const emptyFunctionExpressionProjects = function () {}

const blockArrowProjects = () => {
  return [
    {
      test: {
        name: 'block-arrow',
        include: ['block-arrow/**/*.test.ts'],
      },
    },
  ]
}

const emptyBlockArrowProjects = () => {
  const ignored = []
  return ignored
}

function topLevelFunctionProjects() {
  return [
    {
      test: {
        name: 'top-level-function',
        include: ['top-level-function/**/*.test.ts'],
      },
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
  test: {
    projects: [
      ...parenthesizedProjects,
      ...(true ? [] : []),
      ...unknownProjects,
      ...recursiveIdentifierProjects,
      ...functionExpressionProjects(),
      ...emptyFunctionExpressionProjects(),
      ...blockArrowProjects(),
      ...emptyBlockArrowProjects(),
      ...topLevelFunctionProjects(),
      ...emptyTopLevelFunctionProjects(),
      ...returnOnlyProjects(),
      ...recursiveCallProjects(),
      ...namedVarProjects,
      ...namedFunctionProjects(),
      ...localAliasProjects,
      ...localFunctionProjects,
      ...importedNamedProjects,
      ...sourcedReexportProjects,
      ...cycleProjects,
      ...destructuredProjects,
      ...badProjects,
      ...missingLocalProjects,
      ...noMatchingDeclaration,
      ...missingFileProjects,
      ...packageProjects,
      ...edge.namespaceProjects,
      ...edge.namespaceCallProjects(),
      ...edge.missingNamespaceProjects,
      ...edge.missingNamespaceProjects(),
      ...(true ? edge.namespaceCallProjects : edge.namespaceCallProjects)(),
      ...unknownNamespace.namespaceProjects,
      ...unknownNamespace.namespaceProjects(),
      ...defaultObjectProjects.namespaceProjects,
      ...defaultObjectProjects.namespaceProjects(),
      ...({}).namespaceProjects,
      ...({}).namespaceProjects(),
      ...defaultArrowBlockProjects(),
      ...defaultIdentifierFunctionProjects(),
      ...defaultIdentifierImportedProjects(),
      ...literalDefaultProjects,
      ...missingDefaultProjects,
    ],
  },
})
