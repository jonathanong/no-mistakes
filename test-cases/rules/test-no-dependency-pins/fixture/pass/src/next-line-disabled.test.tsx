const prior = <div>'okay</div>
const example = <pre>actions/checkout@v6.0.2 expect(packageJson.dependencies.foo).toBe('1.2.3')</pre>
const fragment = <>RUN v1.2.3</>

// no-mistakes-disable-next-line test-no-dependency-pins
const view = <div>it's okay</div>; expect(
  getPackageJson().peerDependencies.eslint,
).toBe('>=9')
