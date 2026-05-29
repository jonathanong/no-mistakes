const configs = {
  web: [{ test: { name: 'vitest-destructured-spread-export', include: ['vitest-destructured-spread-export/**/*.test.ts'] } }],
}

export const { web } = { ...configs }
