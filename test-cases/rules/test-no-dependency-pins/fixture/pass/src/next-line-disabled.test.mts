// no-mistakes-disable-next-line test-no-dependency-pins
expect(
  /* misleading expect( and ) delimiters */
  packageJson.devDependencies?.['@typescript-eslint/parser'],
).toEqual(
  '8.42.0',
)
