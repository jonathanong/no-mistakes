# `nextjs-no-api-routes`

## Counterexample

```ts
// pages/api/users.ts
export default function handler() {}
```

## Analysis

### Effort

- Runs at `max` only. Configuring `effort: low` (on the rule, or as a repository or CLI default) is not an error: `check` records the warning `requested effort low; effective effort max because this rule has no low implementation` and runs the rule at `max`.

### Unanalyzable inputs

Following [recall over precision](../../README.md#how-analysis-works), nothing this rule cannot prove is treated as clean, and it has no such inputs.

- This rule has no unanalyzable inputs: it judges file paths and route-handler presence. Nothing is routed through `options.unanalyzable`.

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

```ts
// app/page.tsx
export default function Page() { return null; }
```

## Limitations

The checker flags App Router route handlers and `pages/api` files, but
discovery drops a path when a directory segment is `node_modules`, `dist`,
`.git`, `.next`, `coverage`, `fixtures`, `target`, or `build`.

```text
app/build/route.ts
```

That route handler is not reported.

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
