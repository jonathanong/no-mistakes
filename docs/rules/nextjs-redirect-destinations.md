# `nextjs-redirect-destinations`

## Suppression

Use a line directive for a deliberate external or dynamic destination. Prefer
an `allow` or `ignorePrefixes` entry when that route class is intentional.

## Why and when

Use this rule when rewrites and redirects must remain aligned with real App
Router pages as routes evolve.

## What it catches

It reports static redirect/rewrite destinations that cannot be matched to the
configured route inventory; dynamic destinations remain outside the heuristic.

## Options

`configFiles`, `routeRoots`, `allow`, and `ignorePrefixes` select configuration,
pages, and exceptions. Omitted values use the documented Next.js defaults.

## Valid example

A redirect from `/old` to an existing `app/new/page.tsx` route passes.

## Related rules

[`nextjs-no-api-routes`](nextjs-no-api-routes.md) keeps route conventions
consistent; [`config-path-references`](config-path-references.md) checks static
paths in structured configuration.

Checks that Next.js `redirects()` destinations (and, by default, `rewrites()`)
resolve to real App Router pages. Stale destinations send users to 404s.

The rule does not assume a `web/` project layout. It reads `next.config.ts`,
`next.config.mjs`, or `next.config.js` at the configured root, or a path from
`configPath`. Pages come from `page.{tsx,ts,jsx,js}` under `appRoot` (default
`app`). `_`-prefixed segments are private and do not count as routes.
`(group)` and `@slot` segments unwrap.

```yaml
rules:
  - rule: nextjs-redirect-destinations
    scope: repository
    options:
      configPath: next.config.ts
      appRoot: app
      includeRewrites: true
```

`includeRewrites` defaults to `true`, so rewrite destinations in `beforeFiles`,
`afterFiles`, and `fallback` are checked unless you set `includeRewrites: false`.

External destinations (`://`, `//`) and parameterized `:[A-Za-z]` destinations
are skipped. Query strings and hashes are stripped before matching. Dynamic
App Router segments use the same `[slug]`, `[...x]`, and `[[...x]]` matching
as Filaments `matchesRouteSegments`.

If `redirects` or `rewrites` text exists but the extractor cannot find a
method/function body or string destinations, the rule reports extractor drift
instead of silently passing.

Counterexample: `next.config.ts` redirects `/old` to `/gone`, and
`app/gone/page.tsx` does not exist. A destination of `/secret` also fails when
the only page is `app/_secret/page.tsx`, because `_` segments are private.

Fix: restore the missing `app/**/page.tsx`, point the destination at an
existing route, or remove the stale redirect/rewrite. For private folders,
move the page out of a `_` segment or stop redirecting to that path.

Use `no-mistakes-disable-file nextjs-redirect-destinations` to opt a Next.js
config out, or `no-mistakes-disable-next-line nextjs-redirect-destinations`
on a destination line.
