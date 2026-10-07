# `nextjs-redirect-destinations`

## Suppression

Use a line directive for a deliberate external or dynamic destination. Prefer
the rule-level `exclude` filter when the whole config file is outside policy;
there are no rule-local destination allowlists.

## Why and when

Use this rule when rewrites and redirects must remain aligned with real App
Router pages as routes evolve.

## What it catches

It reports static redirect/rewrite destinations that cannot be matched to the
configured route inventory. If a returned entry cannot be recovered statically,
it reports incomplete extraction alongside any known missing destinations.

## Options

`configPath`, `appRoot`, `includeRewrites`, and `trackedRoutesOnly` are the rule's options.
`configPath` and `appRoot` are optional; when omitted, the rule discovers the
standard `next.config.{ts,mjs,js}` and uses `app`. `includeRewrites` defaults
to `true`. `trackedRoutesOnly` defaults to `false`.

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
      trackedRoutesOnly: false
```

`includeRewrites` defaults to `true`, so rewrite destinations in `beforeFiles`,
`afterFiles`, and `fallback` are checked unless you set `includeRewrites: false`.

Set `trackedRoutesOnly: true` to require pages present in the prepared Git index
inventory. Untracked and ignored pages cannot satisfy literal or tuple-map
destinations, even when present on disk; staging a page with `git add` makes it
eligible on the next invocation. This also applies to rewrites. The default
uses the existing ignore-aware filesystem inventory. Route groups and
dynamic/catch-all matching are identical in both modes. A tracked-route request
without a prepared Git index inventory returns an error instead of falling back
to filesystem pages. Each configured project uses its own prepared Git scope,
so a non-Git umbrella directory can contain Git-backed Next.js projects. The
Rust `run_filesystem_rules_with_files` entrypoint also accepts an authoritative
tracked-file list; generic visible lists require the accompanying discovery
snapshot. Next.js configuration code is never executed.

For example, `/new` is valid with a staged `app/new/page.tsx`; with the option
enabled, the same page left untracked is a missing destination. Fix the finding
by adding the intended page to Git, removing the redirect, or targeting a tracked
route.

External destinations (`://`, `//`) and parameterized `:[A-Za-z]` destinations
are skipped. Query strings and hashes are stripped before matching. Dynamic
App Router segments use the same `[slug]`, `[...x]`, and `[[...x]]` matching
as an application `matchesRouteSegments` helper.

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

## Static destination construction

Immutable module and method-local constants, literal arrays and objects, array
spreads, tuple or object destructuring, and single-parameter synchronous arrow
callbacks to `.map()` are supported. Template strings can interpolate recovered
strings. TypeScript annotations, `as const`, and `satisfies` wrappers are transparent.

```ts
const routePairs = [["/old", "/new"]] as const;
export default {
  async redirects() {
    return [
      { source: "/", destination: "/home", permanent: true },
      ...routePairs.map(([source, target]) => ({
        source,
        destination: `${target}`,
        permanent: true,
      })),
    ];
  },
};
```

Both `/home` and `/new` must match the configured page inventory. Destinations
in unused constants or nested helper bodies do not count as returned entries.

Unknown calls may invoke callbacks or helpers that mutate captured containers,
so they conservatively invalidate known arrays and objects. Uncalled helper
declarations do not invalidate their captured bindings.

Mutation, mutable bindings, unknown spreads, unsupported callbacks, and dynamic
interpolation produce incomplete extraction. Replace them with immutable static
construction, or suppress the finding when runtime behavior is intentional.
An invocation bounds evaluation to 4,096 expression steps per environment or
method, 64 expression levels, and 65,536 bytes per constructed template string;
exceeding a limit also reports incomplete extraction.

Destination collection and mutation alias traversal are also bounded to 4,096
value visits and 64 nesting levels; exhaustion reports incomplete extraction.
