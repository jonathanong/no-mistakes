export const importedBaseProject = {
  testDir: './imported-spread',
  testMatch: ['**/*.imported-spread.spec.ts'],
  testIgnore: ['**/*.imported-spread.skip.ts'],
}

export const importedTrailingBaseProject = {
  name: 'trailing-imported-spread',
  testDir: './trailing-imported-spread',
  testMatch: ['**/*.trailing-imported-spread.spec.ts'],
  testIgnore: ['**/*.trailing-imported-spread.skip.ts'],
}

export const reexportedBaseProject = {
  testDir: './reexported-spread',
  testMatch: ['**/*.reexported-spread.spec.ts'],
}

export const namespaceBaseProject = {
  testDir: './namespace-spread',
  testMatch: ['**/*.namespace-spread.spec.ts'],
}

const localAliasedBaseProject = {
  testDir: './local-alias-spread',
  testMatch: ['**/*.local-alias-spread.spec.ts'],
}

export { localAliasedBaseProject as localAliasBaseProject }
