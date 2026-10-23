# `nextjs-redirect-destinations`

## Counterexample

```ts
// next.config.ts
export default { async redirects() { return [{ source: '/a', destination: '/gone', permanent: true }]; } };
```

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

A redirect from `/a` to an existing `app/b/page.tsx` route passes.

```ts
// next.config.ts
export default { async redirects() { return [{ source: '/a', destination: '/b', permanent: true }]; } };

// app/b/page.tsx
export default function Page() { return null; }
```


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
inventory. Untracked pages, including those excluded by ignore rules, cannot
satisfy literal or tuple-map
destinations, even when present on disk; staging a page with `git add` makes it
eligible on the next invocation. Index membership also includes sparse-checkout
pages not materialized locally and tracked pages removed only from the worktree;
removing a page from the index makes it ineligible. Configuration source must
still be locally readable. This also applies to rewrites. The default
uses the existing ignore-aware filesystem inventory. Route groups and
dynamic/catch-all matching are identical in both modes. A tracked-route request
without a prepared Git index inventory returns an error instead of falling back
to filesystem pages: a local page cannot prove index membership. Run the check
against a Git-backed configured project, provide an authoritative tracked list
through Rust `run_filesystem_rules_with_files`, or set `trackedRoutesOnly: false`
to explicitly use filesystem routes. Each configured project uses its own prepared Git scope,
so a non-Git umbrella directory can contain Git-backed Next.js projects. Even
overlapping project directories keep separate tracked route sets. Sibling
projects in one nested Git repository share its prepared index discovery.
Ignored-but-staged project configs are read through the request source store. The
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

A config entry that does not itself contain `redirects` or `rewrites` is
followed one hop when it re-exports a relative default (`export { default }
from "./next-shared"`) or default-imports that binding and exports it
(`import nextConfig from "./next-shared"; export default nextConfig`). Every
default import is remembered and matched to the exported identifier, including
when another default import comes later. A specifier counts as having an
extension only when that extension is a JS or TS source extension. A dotted
basename such as `./next.shared` still receives those extensions, including
`next.shared.ts` and `next.shared/index.ts`. A `.js` / `.jsx` / `.mjs` /
`.cjs` specifier also resolves the TypeScript sibling. The shared file is then
checked with the same extractor. A second hop, a package specifier, a tsconfig
path such as `@/next-shared`, `export * from`, `module.exports = require("./x")`,
and a wrapper such as `export default withPlugin(shared)` or `{ ...shared }`
are not followed. Two entries that delegate to the same shared module each
report that module's findings.

Counterexample: `next.config.ts` redirects `/old` to `/gone`, and
`app/gone/page.tsx` does not exist. A destination of `/secret` also fails when
the only page is `app/_secret/page.tsx`, because `_` segments are private.

Fix: restore the missing `app/**/page.tsx`, point the destination at an
existing route, or remove the stale redirect/rewrite. For private folders,
move the page out of a `_` segment or stop redirecting to that path.

Use `no-mistakes-disable-file nextjs-redirect-destinations` to opt a Next.js
config out, or `no-mistakes-disable-next-line nextjs-redirect-destinations`
on a destination line.

## Effort

`effort: max` is the default and preserves helper, binding, map-callback, and
template inference described below. At effective `effort: low`, this rule
accepts only literal arrays and objects returned from `redirects()` /
`rewrites()`:

```ts
export default {
  async redirects() {
    return [{ source: "/a", destination: "/b", permanent: true }];
  },
};
```

String literals, no-expression templates, and function bodies that only return
that literal array qualify. Identifiers, helpers, `.map()` callbacks, spreads,
destructuring, and interpolated templates are **unanalyzable**, even when `max`
can recover them. Rewrite them as a literal array of objects with string
`destination` fields, for example
`return [{ source: '/a', destination: '/b', permanent: true }]`.

Missing pages for recovered literal destinations remain ordinary findings at
both efforts, including when the same `redirects()` / `rewrites()` array also
has unanalyzable entries. `unanalyzable` only governs the undecidable rows.

The shared `unanalyzable` policy controls unsupported shapes: `report` fails,
`advise` emits a non-failing advisory, and `ignore` skips them. For example,
set `options.effort: low` and `options.unanalyzable: report` to require literal
arrays. Shared configuration and CLI overrides also apply. These syntax proofs
do not require a companion rule.

Low-effort unanalyzable diagnostics use the `redirects` / `rewrites` method
line. Suppress at that method:

```ts
// no-mistakes-disable-next-line nextjs-redirect-destinations
async redirects() {
  return rows.map((row) => row);
}
```

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

## Delegated configuration

The default config binding may import or re-export one local module through a
relative specifier or a configured TypeScript `paths` alias. The rule uses the
request's prepared project resolver and attributes findings to the shared
module. For example, `import nextConfig from "@/next-shared"; export default
nextConfig;` checks `next-shared.ts` when the selected tsconfig maps `@/*` to
`./*`. A `/gone` redirect there still requires a matching App Router page.

Only one hop is followed. Package imports, `baseUrl`-only specifiers, and a
second delegated module are outside this bounded extraction. Keep redirects
and rewrites in the directly delegated module to make them statically checkable.

At either effort, the same one-hop relative re-export or configured TypeScript paths alias is followed. Low effort checks literal destinations in the delegated file and applies `unanalyzable` to its unsupported entries. Findings and advisories name that helper file, including its suppression directives. For rewrites, use `return [{ source: "/a", destination: "/b" }]` (no `permanent` field).
