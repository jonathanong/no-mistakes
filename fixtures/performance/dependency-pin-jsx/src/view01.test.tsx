// Saved JSX assertions exercise text masking, fragments, and code containers.
import React from 'react';
const packageJson = { dependencies: { react: '^18.0.0' } };
const view01 = (
  <section>
    <h1>Dependency manifest example 1</h1>
    <pre>"expect(packageJson.dependencies.react).toBe('18.2.0')</pre>
    <div>
      expect(packageJson.dependencies.react).toEqual('18.2.0')
      {expect(packageJson.dependencies.react).toBe('18.2.0')}
    </div>
    <>
      <span>{expect(packageJson.dependencies.react).toEqual('18.2.0')}</span>
      <p>Ordinary Unicode text — no assertion here.</p>
    </>
  </section>
);
expect(packageJson.dependencies.react).toBe('18.2.0');
expect(packageJson.dependencies.react).toEqual('18.2.0');
const quoted = "expect(packageJson.dependencies.react).toBe('18.2.0')";
// expect(packageJson.dependencies.react).toEqual('18.2.0')
export default view01;
