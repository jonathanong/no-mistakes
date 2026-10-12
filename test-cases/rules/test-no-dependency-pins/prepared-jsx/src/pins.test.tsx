// Text that looks like an assertion is not code, even with an unmatched quote.
const view = <pre>"expect(packageJson.dependencies.foo).toBe('1.2.3')</pre>;
expect(packageJson.dependencies.foo).toBe('1.2.3');
const fragment = <>{expect(packageJson.devDependencies.bar).toBe('4.5.6')}</>;
// no-mistakes-disable-next-line test-no-dependency-pins -- fixture suppression
expect(packageJson.dependencies.suppressed).toBe('7.8.9');
const quoted = "expect(packageJson.dependencies.foo).toBe('1.2.3')";
// expect(packageJson.dependencies.foo).toBe('1.2.3')
const template = `expect(packageJson.dependencies.foo).toBe('1.2.3')`;
const custom = 'CUSTOM_PIN';
