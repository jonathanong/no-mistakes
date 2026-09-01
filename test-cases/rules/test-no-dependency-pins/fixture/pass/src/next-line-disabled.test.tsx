const prior = <div>'okay</div>

// no-mistakes-disable-next-line test-no-dependency-pins
const view = <div>it's okay</div>; expect(
  getPackageJson().peerDependencies.eslint,
).toBe('>=9')
