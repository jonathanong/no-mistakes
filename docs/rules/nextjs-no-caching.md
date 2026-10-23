# `nextjs-no-caching`

## Counterexample

```ts
// app/page.tsx
const mode = 'force-cache';
export default async function Page() {
  for (const mode of ['no-store'] as const) {
    await fetch('/api/data', { cache: mode });
  }
  // The loop binding ended, so this reads the outer force-cache constant.
  await fetch('/api/data', { cache: mode });
  return null;
}
```

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

There are no rule-local options. The shared `options.effort` and
`options.unanalyzable` settings apply. The rule
uses the selected project and the shared rule
`include`/`exclude` path filters; its Next.js cache patterns are fixed by the
analyzer.

## Effort

`max` is the default and preserves same-file constant resolution and configuration
wrapper analysis. `low` checks literal settings directly at the use site:

```yaml
rules:
  - rule: nextjs-no-caching
    projects: [web]
    options:
      effort: low
      unanalyzable: advise
```

At low effort, cache directives and Next.js cache API imports/calls
remain checked. Fetch options must be an inline object, including an inline
`next` object; route segment values must be literal exports; `next.config` must
export an object directly (or assign one directly to `module.exports`). For
scalar values, strings, numbers, booleans, null, and expression-free templates
are literal; TypeScript scalar wrappers are unwrapped without inference. For
example, both efforts report these literal violations:

```ts
export const revalidate = 60;
await fetch('/api/data', { cache: 'force-cache' });
```

Low effort does not propagate constants or infer configuration through wrappers:

```ts
const cacheMode = 'force-cache';
await fetch('/api/data', { cache: cacheMode });
```

This indirect option is unanalyzable at low effort. Rewrite it as a literal
option at the call, or select max to resolve the constant. Unsupported option
objects, spreads, computed keys, indirect segment exports and configuration
wrappers follow the shared `unanalyzable` policy: `report` fails (the default),
`advise` emits a non-failing advisory, and `ignore` omits the residual diagnostic.
Definite literal violations remain findings under every policy. Max can recover
additional violations from constants; low with `report` can additionally report
uncertainty that max resolves clear, so total counts are not interchangeable.
Spread fetch arguments and destructured route segment exports are also
unanalyzable; write positional fetch arguments and direct literal exports.

These literal proofs need no partner check: unsupported shapes stay explicit
under the selected policy. Low-only runs skip the max constant and configuration
binding tables. Mixed low/max applications share the source inventory and parse,
then use their respective facts.

## Valid example

A selected server component with no cache directive, cached fetch, or configured
cache helper passes.

```ts
// app/page.tsx
export const dynamic = 'force-dynamic';
const mode = 'force-cache';
export default async function Page() {
  try {
    throw 'no-store';
  } catch (mode) {
    if (mode === 'no-store') {
      await fetch('/api/data', { cache: mode });
    }
  }
  return null;
}
```

## Limitations

Both efforts retain the existing fetch identification behavior: a top-level
import or variable named `fetch` excludes the file's fetch calls. Nested local
bindings named `fetch` are not distinguished from the global API; use another
name or a narrow suppression for such a local function.

Discovery drops a file when a directory segment is `node_modules`, `dist`,
`.git`, `.next`, `coverage`, `fixtures`, `target`, or `build`. A caching
directive that lives only there is not reported.

```tsx
// app/build/page.tsx
export const revalidate = 60;
```

Same-file resolution follows `const` bindings, including one `const` that
copies another, and expression-free template keys and values. A `let` or `var`
binding, a computed key whose name is an identifier (`{ [key]: "force-cache" }`),
a template with an expression (`` `force-${kind}` ``), and a binding imported
from another module are left alone. A `const` declared in a nested block is
not visible after that block.

Catch parameters and function or class declaration names hide an outer
constant in their own lexical scope. A named function or class expression
binds its name only inside that expression. `var` names hide an outer constant
throughout their nearest function or class static block, even when declared
inside a nested block. They do not cross a nested function, arrow, or class.
Declarations are collected before resolving aliases, so a later declaration
also hides the outer name in an earlier nested function or alias initializer.
Function body declarations do not hide a constant read by a default parameter
initializer; parameter bindings still hide outer constants there.

`for`, `for-of`, and `for-in` lexical bindings apply to the head and body, then
end after the loop. `switch` case tests and statements share one lexical scope;
the discriminant is evaluated in the surrounding scope first. A case binding
does not hide an outer constant after the switch. A `var` in either statement
still belongs to its nearest function or static block.

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

Counterexample: `fetch(url, { cache: "force-cache" })`. The same fetch option is
reported when the key or value is an expression-free template
(`` [`cache`]: `force-cache` ``) or a same-file `const` alias
(`const mode = "force-cache"`), including a `const` declared later in the same
block and read by an earlier nested function. A parameter, `let`, `var`, or
other non-literal binding with the same name hides an outer `const`. `let`,
`var`, computed identifier keys, templates with expressions, a block-scoped
`const` used after that block, and aliases imported from another file stay
unreported.

Fix: remove caching or isolate it behind an explicitly allowed architecture.
