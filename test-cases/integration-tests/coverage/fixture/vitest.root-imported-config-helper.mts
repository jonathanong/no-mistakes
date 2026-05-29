export const sharedRootConfig = {
  test: {
    projects: [
      {
        test: {
          name: 'root-imported-test-projects',
          include: ['root-imported-test-projects/**/*.test.ts'],
        },
      },
    ],
  },
}

const aliasRootConfig = {
  test: {
    projects: [
      {
        test: {
          name: 'root-alias-imported-test-projects',
          include: ['root-alias-imported-test-projects/**/*.test.ts'],
        },
      },
    ],
  },
}

export { aliasRootConfig as sharedAliasRootConfig }
