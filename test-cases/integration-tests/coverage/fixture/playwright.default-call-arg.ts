const projects = (name: string) => [
  {
    name,
    testMatch: [`${name}/**/*.spec.ts`],
  },
]

export default projects('pw-default-call-arg')
