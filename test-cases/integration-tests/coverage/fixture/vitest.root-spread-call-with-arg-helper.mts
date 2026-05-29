export function makeConfig(x: number) {
  return {
    test: {
      projects: [
        {
          test: {
            name: `project-${x}`,
            include: [`project-${x}/**/*.test.ts`],
          },
        },
      ],
    },
  }
}
