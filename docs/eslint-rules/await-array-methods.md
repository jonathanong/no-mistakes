# `no-mistakes/await-array-methods`

Disallows awaiting synchronous array methods.

Why: awaiting `map`, `filter`, `forEach`, and similar methods usually hides a
missing `Promise.all()` or a mistaken async control flow.

Counterexample: `await items.map(async item => save(item))`.

Fix: remove the `await`, or wrap async mapped work in `Promise.all()`.
