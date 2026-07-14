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
import type { HelperType } from './helpers'
import { type OtherHelperType } from './helpers'

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

  function restParameterShadow(
    ...aside: Array<(page: unknown, testId: string) => unknown>
  ) {
    aside(page, 'shadowed-button')
  }
  restParameterShadow()

  const arrowRestParameterShadow = (...defaultLocator: unknown[]) => {
    defaultLocator('shadowed-button')
  }
  arrowRestParameterShadow()

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

  for (let aside = () => undefined; false; ) {
    aside(page, 'shadowed-button')
  }

  for (const locators in {}) {
    locators.byTestId(page, 'shadowed-button')
  }

  {
    const { aside, ...locators } = {
      aside: () => undefined,
      byTestId: () => undefined,
    }
    aside(page, 'shadowed-button')
    locators.byTestId(page, 'shadowed-button')
  }

  {
    const [aside = () => undefined, ...locators] = []
    aside(page, 'shadowed-button')
    locators.byTestId(page, 'shadowed-button')
  }

  {
    class aside {}
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
    ;(() => undefined)
    aside(page, 'shadowed-button')
    var aside = (_page: unknown, testId: string) => testId
  }
  void varShadow
  void (0 as unknown as HelperType)
  void (0 as unknown as OtherHelperType)

  findById(page, 'unconfigured-button')
})
