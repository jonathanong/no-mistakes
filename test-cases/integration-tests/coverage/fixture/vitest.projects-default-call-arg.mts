const buildDefaultCallProjects = (name: string) => [
  {
    test: {
      name,
      include: [`${name}/**/*.test.ts`],
    },
  },
]

export default buildDefaultCallProjects('default-call-arg')
