// Helper that exports a single project config as default
// Covers ExportDefaultDeclarationKind::ObjectExpression in default_export_options
export default {
  test: {
    name: 'vitest-default-export-object',
    include: ['vitest-default-export-object/**/*.test.ts'],
  },
}
