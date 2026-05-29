export function makeProject() {
  const include = ['vitest-object-call-import/**/*.test.ts']
  return {
    test: {
      name: 'vitest-object-call-import',
      include,
    },
  }
}
