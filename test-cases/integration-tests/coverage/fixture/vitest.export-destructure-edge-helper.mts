// Nested destructuring binding: covers BindingPattern != BindingIdentifier branch (line 107)
export const { project: { inner } } = {
  project: {
    inner: {
      test: {
        name: 'vitest-export-destructure-inner',
        include: ['vitest-export-destructure-inner/**/*.test.ts'],
      },
    },
  },
}

// Missing key destructuring: covers property_expression_deep None branch (line 114)
export const { missingProject } = {
  someOtherKey: {
    test: {
      name: 'should-not-appear',
      include: ['should-not-appear/**/*.test.ts'],
    },
  },
}
