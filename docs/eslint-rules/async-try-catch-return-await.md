# `no-mistakes/async-try-catch-return-await`

Requires `return await` inside configured async `try`/`catch` handlers.

Why: returning a promise from a `try` block without `await` lets later rejections
bypass the local `catch`, including rate-limit handlers that are meant to
process those failures.

Counterexample: `try { return request(); } catch (error) { handleRateLimit(error); }`.

Example: `try { return await request(); } catch (error) { handleRateLimit(error); }`.

Fix: write `return await request()` or await the promise before returning it.

Configure `targets` with grouped `sourcePatterns` and `calleeNamePatterns` regex
strings for the catch-handler APIs that require local rejection handling.
