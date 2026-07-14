import { test } from '@playwright/test'
import defaultLocator from '@fixture/default-locator'
import * as locators from '@fixture/namespace-locators'
import { getAsideLocator as aside } from './helpers'
import { ambiguousLocator } from './helpers'
import { getAsideLocator as importedAside } from '#selector-helpers'
import { workspaceLocator } from '@fixture/workspace-locators/aside'
import { missingImportLocator } from '#missing-helper'
import { missingWorkspaceLocator } from '@fixture/workspace-locators/missing'
import { findById } from './unconfigured'

test('configured selector wrappers', async ({ page }) => {
  aside(page, 'aside-button', 'mode')
  defaultLocator('default-button')
  locators.byTestId(page, `namespace-button`)
  locators.getByTestId(page, 'namespace-native-name')
  locators.ambiguousByTestId(page, 'ambiguous-namespace-button')
  importedAside(page, 'package-import-button')
  workspaceLocator(page, 'workspace-export-button')
  ambiguousLocator(page, 'ambiguous-button')
  missingImportLocator(page, 'recognized-missing-button')
  missingWorkspaceLocator(page, 'recognized-missing-button')

  ;(aside)(page, /regex-only/i)
  aside(page, String('dynamic-only'))
  aside()

  // Every local binding below intentionally shadows a configured import.
  function parameterShadow(aside: (page: unknown, testId: string) => unknown) {
    aside(page, 'shadowed-button')
  }
  parameterShadow(() => undefined)

  {
    const defaultLocator = (testId: string) => testId
    defaultLocator('shadowed-button')
  }

  try {
    throw locators
  } catch (locators) {
    // Argument zero is literal so the native getByTestId heuristic must also
    // respect that this configured namespace binding is shadowed.
    locators.getByTestId('shadowed-button', page)
  }

  for (const aside of [() => undefined]) {
    aside(page, 'shadowed-button')
  }

  switch ('shadow') {
    default: {
      const defaultLocator = (testId: string) => testId
      defaultLocator('shadowed-button')
    }
  }

  const recursive = function defaultLocator() {
    defaultLocator('shadowed-button')
  }
  const Named = class locators {
    method() {
      locators.byTestId(page, 'shadowed-button')
    }
  }
  void recursive
  void Named

  function varShadow() {
    aside(page, 'shadowed-button')
    var aside = (_page: unknown, testId: string) => testId
  }
  void varShadow

  findById(page, 'unconfigured-button')
})
