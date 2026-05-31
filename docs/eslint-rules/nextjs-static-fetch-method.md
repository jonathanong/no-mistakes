# `no-mistakes/nextjs-static-fetch-method`

Requires static `fetch()` method options.

Why: route and fetch analysis can only reason about literal HTTP methods.

Counterexample: `fetch("/api/users", { method })`.

Fix: use literal `method` values in fetch options or refactor dynamic method
selection outside code that must be statically traced.
