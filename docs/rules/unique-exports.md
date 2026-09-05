# `unique-exports`

Prevents ambiguous duplicate public export names.

```yaml
rules:
  - rule: unique-exports
    projects: [web]
    options:
      uniqueAcrossTypesAndValues: true
```

Counterexample: two files in the same checked scope both export `Button`.

Next.js App Router / Pages convention exports (`metadata`, `generateMetadata`,
HTTP method handlers on `route.ts`, and similar) are exempt only inside a
detected Next.js project. Remix route-module exports (`loader`, `action`,
`clientLoader`, `clientAction`, `meta`, `links`, `ErrorBoundary`) are exempt
only for files under a configured `type: remix` project's `app/routes/` (or
`app/root.*`). A `@remix-run/*` package.json dependency is not enough.

Fix: rename one export, narrow the rule scope, or use a documented suppression
for intentional public aliases.

## Why and when

Use this rule when agents search for an API by name, find one file, and
recreate the same export in another path. Grep and a single-file linter do
not see the other `getCurrentUser`. Enable it in a shared workspace so each
public name has one definition the agent can reuse or rename.

## What it catches/requires

Two analyzed files must not define the same public export name in the checked
scope, except for documented framework convention exports described above.

## Options and defaults

`uniqueAcrossTypesAndValues` defaults to `false`; set it to `true` when type and
value namespaces must also be unique. Scope comes from the rule application's
projects/tests filters.

## Valid example

```ts
// users.ts
export function getUser() {}
// teams.ts
export function getTeam() {}
```

## Counterexample

```ts
// users.ts and teams.ts both export function getCurrentUser() {}
```

## Fix

Rename one export, narrow the project scope, or deliberately publish one alias
through an approved barrel rather than defining duplicate names.

## Suppression

Use `no-mistakes-disable-file unique-exports` for a framework-generated or
intentional alias file, with the public ownership reason in the comment.

## Related rules

[`required-entrypoint-reachability`](required-entrypoint-reachability.md)
checks runtime registration; [`tsconfig-alias-folder-mapping`](tsconfig-alias-folder-mapping.md)
checks import alias targets.
