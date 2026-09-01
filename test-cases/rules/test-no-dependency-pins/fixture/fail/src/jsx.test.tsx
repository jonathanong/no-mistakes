const view = (
  <pre>
    expect(packageJson.dependencies.foo).toBe('1.2.3')
    {expect(packageJson.dependencies.foo).toBe('1.2.3')}
  </pre>
)
const nested = (
  <section>
    actions/checkout@v6.0.2
    <pre>{expect(packageJson.devDependencies.eslint).toBe('10.9.0')}</pre>
  </section>
)
