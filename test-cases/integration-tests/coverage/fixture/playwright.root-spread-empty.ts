import { defineConfig } from '@playwright/test'
import {
  destructuredConfig,
  missingConfig,
  specifierConfig,
} from './playwright.root-spread-empty-helper'
import { packageConfig } from 'missing-package'
import { unreadableConfig } from './playwright.unreadable'

function configFactory() {
  return {}
}

export default defineConfig({
  ...unknownConfig,
  ...configFactory(),
  ...packageConfig,
  ...unreadableConfig,
  ...missingConfig,
  ...specifierConfig,
  ...destructuredConfig,
})
