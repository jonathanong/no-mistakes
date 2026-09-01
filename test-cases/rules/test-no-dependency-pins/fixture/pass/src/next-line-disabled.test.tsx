const prior = <div>'okay</div>
const example = <pre>actions/checkout@main expect(packageJson.dependencies.foo).toBe('1.2.3')</pre>
const fragment = <>RUN latest</>

// no-mistakes-disable-next-line test-no-dependency-pins
const view = <div>it's okay</div>; expect(
  getPackageJson().peerDependencies.eslint,
).toBe('>=9')
