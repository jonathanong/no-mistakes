import { defineConfig } from '@playwright/test'
// @ts-ignore
import { makeConfig } from './playwright.root-call-unreadable-helper'

// The helper is a directory (unreadable as file), covering the Err(_) path
// in root_spreads/calls.rs imported_project_options (line 49).
export default defineConfig({
  // @ts-ignore
  ...makeConfig(),
})
