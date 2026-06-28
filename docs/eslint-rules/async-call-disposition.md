# `no-mistakes/async-call-disposition`

Requires configured async calls to be explicitly handled.

Why: queue enqueue and job scheduling APIs usually return promises. Floating
those promises can hide failed job submissions and make worker control flow
ambiguous.

Counterexample: `enqueueEmail(user.id)`.

Example:

```js
await enqueueEmail(user.id);
void enqueueEmail(user.id);
return enqueueEmail(user.id);
```

Fix: use `await enqueueEmail(...)`, `return enqueueEmail(...)`,
`void enqueueEmail(...)`, or group multiple calls under an awaited or returned
`Promise.all(...)`.

Configure `targets` with grouped `sourceSpecifierPatterns` and `calleeNamePatterns` glob or
`/regex/` strings so only project-specific async APIs are checked.
