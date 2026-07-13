export const vitestProjects = [
  {
    test: {
      name: 'unit',
      include: ['src/**/*.test.ts'],
    },
  },
]

// Intentionally shared with the Playwright config: the request must parse this
// helper once even though both runner-config evaluators import it.
export const playwrightProjects = [
  {
    name: 'pw-unit',
    testDir: 'playwright',
    testMatch: '**/*.test.ts',
  },
]
