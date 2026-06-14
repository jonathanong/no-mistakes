export function backendCoreProjects(flavor: string) {
  return [
    {
      test: {
        name: `backend-core-${flavor}`,
        include: [`backend/**/*.${flavor}.test.ts`],
      },
    },
  ]
}
