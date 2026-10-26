# `nextjs-no-api-routes`

Bans Next.js API route files beneath the rule's resolved project roots.

```yaml
rules:
  - rule: nextjs-no-api-routes
    projects: [web]
```

## Why and when

Use this rule when server endpoints belong in a separate backend or server
project rather than the Next.js application.

## What it catches

It reports App Router `route` files with supported JS/TS extensions under
`app/` or `src/app/`, and files under `pages/api/` or `src/pages/api/`.
The check judges file paths and needs no AST parse of its own, so malformed
route source still reports when no other configured check requires a parse.

The runner limits this rule's raw-source reads and retention to API-route
candidates beneath the resolved roots. Candidates excluded by a later rule
filter still supply source to preserve the aggregate check's existing
behavior. Mixed checks retain their own source and parse demand, including
source needed to suppress React parse errors and graph source facts.

## Options

Use the common `projects` or `scope` selection and `include` / `exclude`
filters. This rule has no local `roots` or `allow` options.

## Valid example

`app/users/page.tsx` passes because it is a page rather than an API endpoint.

```tsx
// app/users/page.tsx
export default function Page() { return null; }
```

## Counterexample

```ts
// pages/api/users.ts
export default function handler() {}
```

`app/api/users/route.ts` also reports.

## Fix

Move API behavior to the configured backend or server project.

## Analysis

### Effort

- Runs at `max` only. Configuring `effort: low` (on the rule, or as a repository or CLI default) is not an error: `check` records the warning `requested effort low; effective effort max because this rule has no low implementation` and runs the rule at `max`.

### Unanalyzable inputs

Following [recall over precision](../../README.md#how-analysis-works), nothing this rule cannot prove is treated as clean, and it has no such inputs.

- This rule has no unanalyzable inputs: it judges file paths and route-handler presence. Nothing is routed through `options.unanalyzable`.

## Suppression

Use `// no-mistakes-disable-file nextjs-no-api-routes` for a deliberate legacy
endpoint during migration. Findings are on line 1, so a matching
`no-mistakes-disable-line` or `no-mistakes-disable-next-line` directive must
suppress that line.

## Limitations

Discovery drops a path when a directory segment is `node_modules`, `dist`,
`.git`, `.next`, `coverage`, `fixtures`, `target`, or `build`.
For example, `app/build/route.ts` is not reported.

## Related rules

[`server-route-client-boundary`](server-route-client-boundary.md) protects
server-route imports; [`nextjs-redirect-destinations`](nextjs-redirect-destinations.md)
checks App Router destinations.
