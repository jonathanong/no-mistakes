export function webProjects({ root }: { root: string }) {
  return [
    {
      test: {
        name: 'web',
        include: [`${root}/**/*.test.ts`],
        exclude: [`${root}/**/*.generated.test.ts`],
      },
    },
  ]
}
