export const reexportedProjects = [
  {
    test: {
      name: 'reexported',
      include: ['reexported/**/*.test.ts'],
    },
  },
]

export const typedReexportProjects = [
  {
    test: {
      name: 'typed-reexport-runtime',
      include: ['typed-reexport-runtime/**/*.test.ts'],
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
