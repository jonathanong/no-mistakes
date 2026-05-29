export function makeConfig(x: number) {
  return {
    projects: [{ name: `project-${x}`, testMatch: [`project-${x}/**/*.spec.ts`] }],
  }
}
