# `no-mistakes/no-global-fetch-outside-helper`

Disallows direct global `fetch` calls outside configured helper paths.

Why: projects that centralize request behavior in API/client helpers can keep
auth, retry, error, and observability policy in one layer instead of spreading
bare network calls through application code.

Example: a configured helper path centralizes global `fetch`.

```ts
export function getUser(id: string) {
  return fetch(`/api/users/${id}`);
}
```

Counterexample: a checked application file calls global `fetch` directly.

```ts
export function UserPage({ id }: { id: string }) {
  return fetch(`/api/users/${id}`);
}
```

Fix: move the call into a configured helper path and call that helper from the
application file.

```ts
import { getUser } from "../lib/api/users";

export function UserPage({ id }: { id: string }) {
  return getUser(id);
}
```

Options: `checkedPathPatterns` and `allowedPathPatterns`. Both are required and
must be non-empty.
