export const reexportedProjects = [
  {
    test: {
      name: 'reexported',
      include: ['reexported/**/*.test.ts'],
    },
  },
]

const aliasDefaultProjects = [
  {
    test: {
      name: 'alias-default',
      include: ['alias-default/**/*.test.ts'],
    },
  },
]

export default aliasDefaultProjects
