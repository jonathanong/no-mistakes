const configs = {
  web: [{ name: 'pw-destructured-spread-export', testMatch: 'pw-destructured-spread-export/**/*.spec.ts' }],
}

export const { web } = { ...configs }
