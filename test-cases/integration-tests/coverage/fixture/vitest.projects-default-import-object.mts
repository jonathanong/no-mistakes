import { defineConfig } from 'vitest/config'
import projectConfig from './vitest.default-export-object-helper'

// Uses a default-imported object as the projects value
// Covers ExportDefaultDeclarationKind::ObjectExpression in default_export_options
export default defineConfig({
  test: {
    projects: projectConfig,
  },
})
