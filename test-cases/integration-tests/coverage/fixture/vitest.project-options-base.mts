export const importedTestOptions = {
  include: ['imported-nested-test-spread/**/*.test.ts'],
  exclude: ['imported-nested-test-spread/**/*.skip.ts'],
}

export const namedImportedTestOptions = {
  name: 'imported-options-name',
  include: ['imported-options-name/**/*.test.ts'],
}

export default {
  name: 'default-imported-options',
  include: ['default-imported-options/**/*.test.ts'],
}
