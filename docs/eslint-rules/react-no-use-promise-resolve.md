# `no-mistakes/react-no-use-promise-resolve`

Disallows `React.use(Promise.resolve(...))`.

Why: wrapping sync values in resolved promises creates unnecessary async render
shapes.

Counterexample: `React.use(Promise.resolve(user))`.

Fix: pass real asynchronous resources to `React.use()` or keep synchronous data
synchronous.
