// Nested ObjectPattern in binding_property.value: covers declarations.rs:107 (continue)
// When looking for export 'inner', the binding_property for 'project' has a nested
// ObjectPattern value ({ inner }), which is NOT a BindingIdentifier.
export const { project: { inner } } = {
  project: {
    inner: {
      test: {
        name: 'vitest-destructure-nested',
        include: ['vitest-destructure-nested/**/*.test.ts'],
      },
    },
  },
}

// Missing key in source object: covers declarations.rs:114 (continue)
// When looking for export 'aliased', key 'notHere' maps to BindingIdentifier 'aliased',
// but 'notHere' does not exist in the source object → property_expression_deep returns None.
export const { notHere: aliased } = {
  present: {
    test: {
      name: 'vitest-destructure-aliased',
      include: ['vitest-destructure-aliased/**/*.test.ts'],
    },
  },
}
