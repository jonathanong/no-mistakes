import { defineConfig } from 'vitest/config'
import { makeConfig } from './vitest.root-call-import-non-fn-helper'

export default defineConfig({
  ...makeConfig(),
})
