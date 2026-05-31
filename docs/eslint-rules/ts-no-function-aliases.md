# `no-mistakes/ts-no-function-aliases`

Disallows function wrappers that only alias another function.

Why: alias wrappers add indirection without behavior and make call graphs harder
to follow.

Counterexample: `export function getUser(id) { return fetchUser(id); }`.

Fix: import and call the original function directly, or add meaningful behavior
to the wrapper.
