import { defineConfig } from '@playwright/test'
import defaultAsConfig from './playwright.root-spread-default-as'
import defaultNonNullConfig from './playwright.root-spread-default-non-null'
import defaultSatisfiesConfig from './playwright.root-spread-default-satisfies'
import defaultTypeAssertionConfig from './playwright.root-spread-default-type-assertion'
import { cycleConfig } from './playwright.root-spread-cycle-a'
import {
  reexportedSourcedConfig,
  sourcedConfig,
} from './playwright.root-spread-empty-helper'

const recursiveConfig = {
  ...recursiveConfig,
}

export default defineConfig({
  method() {},
  ['ignored']: true,
  ...recursiveConfig,
  ...defaultAsConfig,
  ...defaultNonNullConfig,
  ...defaultSatisfiesConfig,
  ...defaultTypeAssertionConfig,
  ...cycleConfig,
  ...sourcedConfig,
  ...reexportedSourcedConfig,
})
