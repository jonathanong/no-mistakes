// Oxc 0.143 keeps block-arrow bodies distinct from concise expression bodies.
export default () => {
  return [
    { test: { name: 'imported-block-arrow', include: ['block/**/*.test.ts'] } },
  ]
}
