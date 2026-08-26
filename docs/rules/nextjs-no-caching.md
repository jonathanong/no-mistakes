# `nextjs-no-caching`

## Suppression

Use a line directive for a documented cache escape hatch or a file directive
for a route intentionally outside the policy. Prefer a narrow `allow` entry.

## Why and when

Use this rule when a Next.js application deliberately requires dynamic,
request-time behavior instead of implicit cache semantics.

## What it catches

It reports configured Next.js caching APIs, directives, and fetch/cache options
that would make a selected route or component cacheable.

## Options

There are no rule-local options. Consequently, there are no rule-local
defaults. The rule uses the selected project and the shared rule
`include`/`exclude` path filters; its Next.js cache patterns are fixed by the
analyzer.

## Valid example

A selected server component with no cache directive, cached fetch, or configured
cache helper passes.

## Related rules

[`nextjs-no-api-routes`](nextjs-no-api-routes.md) governs legacy API files;
[`nextjs-redirect-destinations`](nextjs-redirect-destinations.md) validates
route configuration.

Bans Next.js caching features such as cache directives, cache wrappers, and
cache-related fetch/config settings.

```yaml
rules:
  - rule: nextjs-no-caching
    projects: [web]
```

Counterexample: `fetch(url, { cache: "force-cache" })`.

Fix: remove caching or isolate it behind an explicitly allowed architecture.
