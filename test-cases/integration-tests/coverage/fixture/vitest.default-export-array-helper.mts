// Helper that exports a projects array as default
// Covers ExportDefaultDeclarationKind::ArrayExpression in default_export_options
export default [
  {
    test: {
      name: 'vitest-default-export-array',
      include: ['vitest-default-export-array/**/*.test.ts'],
    },
  },
]
