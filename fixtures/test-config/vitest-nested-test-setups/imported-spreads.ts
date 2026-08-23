export const nestedImported = {
  test: {
    name: 'imported-nested-spread',
    include: ['imported-nested-spread/**/*.test.ts'],
    setupFiles: './inner-imported.ts',
    globalSetup: './inner-imported-global.ts',
  },
}

export const outerImported = {
  setupFiles: './outer-imported.ts',
  globalSetup: './outer-imported-global.ts',
}
