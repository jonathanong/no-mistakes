import { defineConfig } from 'vitest/config'
// @ts-ignore
import { makeConfig } from './vitest.root-call-unreadable-helper'

// The helper is a directory (unreadable as file), covering the Err(_) path
// in root_spreads/calls.rs imported_project_options (line 51).
export default defineConfig({
  // @ts-ignore
  ...makeConfig(),
})
