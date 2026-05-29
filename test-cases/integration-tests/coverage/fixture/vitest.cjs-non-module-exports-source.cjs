// A non-assignment ExpressionStatement (call expression) covers line 118:
// commonjs_default_expression returns None when expression is not AssignmentExpression
void 0

// An assignment that's NOT module.exports covers line 124 (return None for non-module.exports)
exports.projects = [
  {
    test: {
      name: 'vitest-cjs-exports-dot',
      include: ['vitest-cjs-exports-dot/**/*.test.ts'],
    },
  },
]
