# `no-mistakes/nextjs-static-fetch-url`

Requires static `fetch()` URL arguments.

Why: Next.js route-to-fetch analysis depends on literal URL paths.

Counterexample: ``fetch(`/api/${resource}`)``.

Fix: use a string literal, static template, or supported static helper shape for
internal API calls.
