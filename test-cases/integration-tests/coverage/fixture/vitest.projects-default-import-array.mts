import { defineConfig } from 'vitest/config'
import projectList from './vitest.default-export-array-helper'

// Uses a default-imported array as the projects value
// Covers ExportDefaultDeclarationKind::ArrayExpression in default_export_options
export default defineConfig({
  test: {
    projects: projectList,
  },
})
