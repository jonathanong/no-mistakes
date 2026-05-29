import { defineConfig } from 'vitest/config'
import defaultAsConfig from './vitest.root-spread-default-as'
import defaultNonNullConfig from './vitest.root-spread-default-non-null'
import defaultSatisfiesConfig from './vitest.root-spread-default-satisfies'
import defaultTypeAssertionConfig from './vitest.root-spread-default-type-assertion'
import { cycleConfig } from './vitest.root-spread-cycle-a'
import {
  reexportedSourcedConfig,
  sourcedConfig,
  specifierConfig,
} from './vitest.root-spread-empty-helper'

const recursiveConfig = {
  test: {
    ...recursiveConfig,
  },
}

export default defineConfig({
  test: {
    method() {},
    ['ignored']: true,
    ...recursiveConfig,
    ...defaultAsConfig,
    ...defaultNonNullConfig,
    ...defaultSatisfiesConfig,
    ...defaultTypeAssertionConfig,
    ...cycleConfig,
    ...({}).projects,
    ...missingNamespace.projects,
    ...specifierConfig.projects,
    ...sourcedConfig,
    ...reexportedSourcedConfig,
  },
})
