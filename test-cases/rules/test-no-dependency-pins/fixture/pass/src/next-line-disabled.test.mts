// no-mistakes-disable-next-line test-no-dependency-pins
expect(
  expect(condition).toBe(true),
  /* misleading expect( and ) delimiters */
  getPackageJson().devDependencies?.['@typescript-eslint/parser'],
).toEqual(
  '8.42.0',
)
