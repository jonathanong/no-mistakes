import { defineConfig } from 'vitest/config'
// @ts-ignore
import { inner, aliased } from './vitest.export-destructure-named-helper'

// Uses spread elements in the projects array so expression_options is called for each.
// This follows the export chain: identifier_options → exported_options_lookup →
// named_export_options → declarator_options → destructured_expression.
// - Spreading 'inner' covers declarations.rs:107 (nested ObjectPattern, not BindingIdentifier)
// - Spreading 'aliased' covers declarations.rs:114 (binding key 'notHere' not in source object)
export default defineConfig({
  test: {
    // @ts-ignore
    projects: [...inner, ...aliased],
  },
})
