# `queries-reexport` fixture

Exercises wildcard (`*`), `default`, and re-export forwarding for the query
commands:

- `mod.ts` — exports `used` and a default `def`.
- `named-barrel.ts` — `export { used } from "./mod"` plus an unrelated local
  `used()` call that must NOT be reported as a call site.
- `star-barrel.ts` — `export * from "./mod"` (recorded under `*`).
- `named-consumer.ts` / `star-consumer.ts` — call `used` through each barrel.
- `ns-consumer.ts` — `import * as m from "./mod"` (recorded under `*`).
- `default-consumer.ts` — `import def from "./mod"` (recorded under `default`).
- `lonely.ts` / `lonely-barrel.ts` — a default seen only by an `export *`
  barrel, which does not forward defaults, so the default is dead.
