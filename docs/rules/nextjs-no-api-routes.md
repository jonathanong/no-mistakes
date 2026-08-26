# `nextjs-no-api-routes`

## Suppression

Use a file directive only for a deliberate legacy API route during migration;
prefer an `allow` entry or moving the endpoint to the configured modern boundary.

## Why and when

Use this rule when an App Router project has chosen route handlers or another
server boundary instead of legacy Pages Router API routes.

## What it catches

It reports configured legacy API-route file paths, not route-handler files.

## Options

`roots` selects legacy route roots and `allow` lists intentional exceptions;
omitted values use the documented Pages Router root defaults.

## Valid example

`app/users/page.tsx` passes because it is a page rather than an App Router
route handler or a Pages Router API route.

## Related rules

[`server-route-client-boundary`](server-route-client-boundary.md) protects
server-route imports; [`nextjs-redirect-destinations`](nextjs-redirect-destinations.md)
checks App Router destinations.

Bans Next.js API route files.

```yaml
rules:
  - rule: nextjs-no-api-routes
    projects: [web]
```

Counterexample: `pages/api/users.ts` or `app/api/users/route.ts`.

Fix: move API behavior to the configured backend/server project.
