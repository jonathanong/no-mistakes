import { defineConfig } from 'vitest/config'
import { makeShared } from './vitest.root-call-import-helper'

export default defineConfig({
  ...makeShared(),
})
