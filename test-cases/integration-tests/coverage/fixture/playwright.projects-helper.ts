const sharedProject = {
  testDir: './imported',
  testMatch: ['**/*.imported.spec.ts'],
  testIgnore: ['**/*.skip.ts'],
}

export const importedPlaywrightProjects = [
  {
    ...sharedProject,
    name: 'imported',
  },
]

export function factoryPlaywrightProjects() {
  return [
    {
      name: 'factory',
      testDir: './factory',
      testMatch: ['**/*.factory.spec.ts'],
    },
  ]
}
