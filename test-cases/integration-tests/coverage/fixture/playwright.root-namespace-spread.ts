import { defineConfig } from '@playwright/test'
import * as shared from './playwright.root-namespace-spread-helper'
import { config as namedConfig } from './playwright.root-namespace-spread-helper'

const rootBase = {
  method() {},
  ['ignored']: true,
  testDir: './root-namespace-spread',
}

export default defineConfig({
  ...({}).config,
  ...missingNamespace.config,
  ...namedConfig.config,
  ...rootBase,
  ...shared.config,
})
