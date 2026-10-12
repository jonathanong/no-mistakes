# Test timeout policy

`no-mistakes/test-timeout-policy` provides helpers for enforcing a maximum
configured Vitest deadline. The policy validates positive finite values up to an
inclusive maximum (30,000 ms by default); it does not clamp values. Test and hook
deadlines are separate. A blocked synchronous event loop still requires an outer
process deadline.

Install `testTimeoutPolicyPlugin` in each owning Node Vitest config and list the
packages that register tests or hooks, including transitive helper packages:

```js
import { testTimeoutPolicyPlugin } from "no-mistakes/test-timeout-policy/plugin";

export default {
  plugins: [testTimeoutPolicyPlugin(30_000, ["@app/test-helpers"])],
};
```

The plugin routes public `vitest` imports through guarded exports and configures
listed packages for inlining. It cannot discover omitted registrar packages, and
global APIs bypass import routing. Use the dedicated `/plugin` entry only in
Vite/Vitest configuration; it imports the optional Vitest peer. The policy entry
itself remains available without Vitest.

For a custom Node runner, pass the absolute resolved path of the final runner
module as the third argument:

```js
plugins: [
  testTimeoutPolicyPlugin(30_000, registrationPackages, {
    runner: absoluteFinalRunnerModule,
  }),
]
```

The module must default-export the final constructor. Browser projects use
`browserTestTimeoutPolicyPlugin` instead. See [Node API exports](node-api.md#runtime-export-surface)
for the package entrypoints and [Vitest configuration](rules/vitest-ci-path-coverage.md)
for related repository test configuration guidance.

## Runner lifecycle

`guardPublicRunner` is for direct construction of the final runner class. It
installs protection after that class's fields initialize; it does not install
protection during subclass initialization. If a runner has derived lifecycle or
context fields, wrap the actual final constructor with the Node plugin's
`runner` option so those fields initialize before protection. Alternatively,
call `protectRunnerTimeouts(runner)` after the final class has initialized its
own fields.

The browser plugin uses a separate adapter around Vitest's native browser
runner. It installs a sealed `onBeforeCollect` hook that protects the runner
context when collection begins; replacing that hook fails closed. This route
supports the native browser runner and its extension fields. A custom browser
runner with its own `onBeforeCollect` field needs an explicit final-constructor
route and is not supported by this native browser adapter.
