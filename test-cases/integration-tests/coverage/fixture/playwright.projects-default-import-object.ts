import { defineConfig } from '@playwright/test'
import projectConfig from './playwright.default-export-object-helper'

// Uses a default-imported object (not array) as the projects value
// Covers ExportDefaultDeclarationKind::ObjectExpression in default_export_options
export default defineConfig({
  projects: projectConfig,
})
