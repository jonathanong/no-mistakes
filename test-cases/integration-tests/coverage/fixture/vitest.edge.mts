import type { missingLocalProjects as typeOnlyDeclarationProjects } from './vitest.edge-source'
import { type noMatchingDeclaration as typeOnlySpecifierProjects } from './vitest.edge-source'
import defaultAsProjects from './vitest.edge-default-as-array'
import defaultArrowBlockProjects from './vitest.edge-default-arrow-block'
import defaultDeclarationOnlyProjects from './vitest.edge-default-declaration-only'
import defaultDirectAsProjects from './vitest.edge-default-direct-as-array'
import defaultDirectObjectProject from './vitest.edge-default-direct-object'
import defaultDirectSatisfiesProjects from './vitest.edge-default-direct-satisfies-array'
import defaultDirectTypeAssertionProjects from './vitest.edge-default-direct-type-assertion-array'
import defaultExportedConstProjects from './vitest.edge-default-exported-const'
import defaultIdentifierFunctionProjects from './vitest.edge-default-identifier-function'
import defaultIdentifierImportedProjects from './vitest.edge-default-identifier-import'
import defaultNonNullProjects from './vitest.edge-default-non-null-array'
import defaultSatisfiesProjects from './vitest.edge-default-satisfies-array'
import defaultTypeAssertionProjects from './vitest.edge-default-type-assertion-array'
import nonSpreadImportedArrayProjects from './vitest.non-spread-imported-array'
import literalDefaultProjects from './vitest.edge-default-literal'
import missingDefaultProjects from './vitest.edge-default-missing'
import * as edge from './vitest.edge-source'
import { ambiguousStarProjects } from './vitest.ambiguous-star-barrel'
import { nonambiguousStarProjects } from './vitest.nonambiguous-star-barrel'
import { typeStarProjects } from './vitest.type-star-barrel'
import defaultObjectProjects from './vitest.projects-source'
import {
  cycleProjects,
  computedDestructuredProjects,
  destructuredProjects,
  exportedSpecifierAliasProjects,
  importedNamedProjects,
  localAliasProjects,
  localFunctionProjects,
  missingLocalProjects,
  namedFunctionProjects,
  namedVarProjects,
  overloadedProjects,
  reexportedProjects,
  sourcedReexportProjects,
  typedReexportProjects,
  noMatchingDeclaration,
} from './vitest.edge-source'
import { badProjects } from './vitest.edge-bad'
import { missingFileProjects } from './vitest.missing-file'
import { packageTestOptions } from 'missing-package'
import defaultTestOptions, {
  importedTestOptions,
  missingImportedTestOptions,
  namedImportedTestOptions,
} from './vitest.project-options-base'
import { unreadableProjects } from './vitest.unreadable'
import { unreadableTestOptions } from './vitest.unreadable'
import { packageProjects } from 'missing-package'
import { projectTestOverride } from './vitest.project-test-spread-base'
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
const recursiveTestOptions = recursiveTestOptions
const nestedLocalTestOptions = {
  test: {
    include: ['nested-local-spread/**/*.test.ts'],
  },
}
const constantInclude = ['project-constant-spread/**/*.test.ts']
const projectConstantBase = {
  test: {
    include: constantInclude,
  },
}
const nestedArrayProjects = [
  {
    test: {
      name: 'nested-array-should-not-flatten',
      include: ['nested-array-should-not-flatten/**/*.test.ts'],
    },
  },
]
const wrappedHelperProjects = (() => [
  {
    test: {
      name: 'wrapped-helper',
      include: ['wrapped-helper/**/*.test.ts'],
    },
  },
]) satisfies () => unknown[]

function nonSpreadCallArrayProjects() {
  return [
    {
      test: {
        name: 'vitest-non-spread-call-array',
        include: ['vitest-non-spread-call-array/**/*.test.ts'],
      },
    },
  ]
}

function objectProject() {
  return {
    test: {
      name: 'vitest-object-call-project',
      include: ['vitest-object-call-project/**/*.test.ts'],
    },
  }
}

const objectArrowProject = () => ({
  test: {
    name: 'vitest-object-call-arrow-project',
    include: ['vitest-object-call-arrow-project/**/*.test.ts'],
  },
})

const objectBlockProject = () => {
  return {
    test: {
      name: 'vitest-object-call-block-project',
      include: ['vitest-object-call-block-project/**/*.test.ts'],
    },
  }
}

const objectFunctionProject = function () {
  return {
    test: {
      name: 'vitest-object-call-function-project',
      include: ['vitest-object-call-function-project/**/*.test.ts'],
    },
  }
}

const objectExpressionProject = {
  test: {
    name: 'vitest-object-call-expression-project',
    include: ['vitest-object-call-expression-project/**/*.test.ts'],
  },
}

const recursiveObjectProject = () => recursiveObjectProject()

function objectNoReturnProject() {
  const ignored = true
}

function objectReturnOnlyProject() {
  return
}

export default defineConfig({
  test: {
    projects: [
      ,
      ...parenthesizedProjects,
      ...(true ? [] : []),
      ...unknownProjects,
      ...recursiveIdentifierProjects,
      ...functionExpressionProjects(),
      ...emptyFunctionExpressionProjects(),
      ...blockArrowProjects(),
      ...emptyBlockArrowProjects(),
      ...topLevelFunctionProjects('ignored'),
      ...topLevelFunctionProjects(),
      ...emptyTopLevelFunctionProjects(),
      ...returnOnlyProjects(),
      ...recursiveCallProjects(),
      ...wrappedHelperProjects(),
      [
        {
          test: {
            name: 'direct-nested-array-should-not-flatten',
            include: ['direct-nested-array-should-not-flatten/**/*.test.ts'],
          },
        },
      ],
      nestedArrayProjects,
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
      ...ambiguousStarProjects,
      ...nonambiguousStarProjects,
      ...typeStarProjects,
      ...namedVarProjects,
      ...namedFunctionProjects(),
      ...overloadedProjects(),
      ...localAliasProjects,
      ...localFunctionProjects,
      ...importedNamedProjects,
      ...sourcedReexportProjects,
      ...typedReexportProjects,
      ...cycleProjects,
      ...destructuredProjects,
      ...exportedSpecifierAliasProjects,
      ...badProjects,
      ...computedDestructuredProjects,
      ...missingLocalProjects,
      ...noMatchingDeclaration,
      ...typeOnlyDeclarationProjects,
      ...typeOnlySpecifierProjects,
      ...missingFileProjects,
      ...unreadableProjects,
      ...packageProjects,
      {
        test: {
          method() {},
          ['ignored']: true,
          ...recursiveTestOptions,
          ...missingLocalTestOptions,
          ...true,
          ...packageTestOptions,
          ...unreadableTestOptions,
          ...defaultTestOptions,
          ...namedImportedTestOptions,
          ...missingImportedTestOptions,
          ...importedTestOptions,
          name: 'imported-nested-test-spread',
        },
      },
      {
        root: 'packages/app',
        test: {
          name: 'vitest-project-root',
          include: ['**/*.test.ts'],
          exclude: ['ignored/**/*.test.ts'],
        },
      },
      {
        ...nestedLocalTestOptions,
        name: 'nested-local-spread',
      },
      {
        test: {
          name: 'project-test-spread-local',
          include: ['project-test-spread-local/**/*.test.ts'],
        },
        ...projectTestOverride,
      },
      {
        test: {
          ...projectConstantBase.test,
          name: 'project-constant-spread',
        },
      },
      {
        test: {
          ...edge.namespaceTestOptions,
          ...({}).missingTestOptions,
          ...unknownNamespaceTestOptions.web,
          ...namedImportedTestOptions.missing,
          name: 'namespace-test-options-spread',
        },
      },
      ...reexportedProjects,
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
      ...defaultAsProjects,
      ...defaultArrowBlockProjects(),
      ...defaultDeclarationOnlyProjects,
      ...defaultDirectAsProjects,
      defaultDirectObjectProject,
      ...defaultDirectSatisfiesProjects,
      ...defaultDirectTypeAssertionProjects,
      ...defaultExportedConstProjects,
      ...defaultIdentifierFunctionProjects(),
      ...defaultIdentifierImportedProjects(),
      ...defaultNonNullProjects,
      ...defaultSatisfiesProjects,
      ...defaultTypeAssertionProjects,
      ...literalDefaultProjects,
      ...missingDefaultProjects,
    ],
  },
})
