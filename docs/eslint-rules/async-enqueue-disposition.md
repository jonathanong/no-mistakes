# `no-mistakes/async-enqueue-disposition`

Requires configured async enqueue calls to be explicitly handled.

Why: queue enqueue APIs usually return promises. Floating those promises can hide
failed job submissions and make worker control flow ambiguous.

Counterexample: `enqueueEmail(user.id)`.

Fix: use `await enqueueEmail(...)`, `return enqueueEmail(...)`,
`void enqueueEmail(...)`, or group multiple calls under an awaited or returned
`Promise.all(...)`.

Configure `targets` with grouped `sourcePatterns` and `calleeNamePatterns` regex
strings so only project-specific enqueue APIs are checked.
