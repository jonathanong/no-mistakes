export default {
  testDir: './root-imported-defaults',
  testMatch: ['**/*.shared.spec.ts'],
  projects: [
    {
      name: 'root-imported-config',
    },
  ],
}

export const namedImportedConfig = {
  projects: [
    {
      name: 'root-named-imported-config',
      testMatch: ['root-named-imported-config/**/*.spec.ts'],
    },
  ],
}

const localAliasImportedConfig = {
  projects: [
    {
      name: 'root-local-alias-imported-config',
      testMatch: ['root-local-alias-imported-config/**/*.spec.ts'],
    },
  ],
}

export { localAliasImportedConfig as aliasImportedConfig }
