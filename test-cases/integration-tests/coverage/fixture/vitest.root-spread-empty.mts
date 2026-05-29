import { defineConfig } from 'vitest/config'
import {
  destructuredConfig,
  missingConfig,
  specifierConfig,
} from './vitest.root-spread-empty-helper'
import { packageConfig } from 'missing-package'
import { unreadableConfig } from './vitest.unreadable'

function configFactory() {
  return {}
}

export default defineConfig({
  test: {
    ...unknownConfig,
    ...configFactory(),
    ...packageConfig,
    ...unreadableConfig,
    ...missingConfig,
    ...specifierConfig,
    ...destructuredConfig,
  },
})
