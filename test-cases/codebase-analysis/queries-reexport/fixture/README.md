# `queries-reexport` fixture

Exercises wildcard (`*`) and `default` index symbols for the query commands:

- `mod.ts` — exports `used` and a default `def`.
- `star-barrel.ts` — `export * from "./mod"` (recorded under `*`).
- `ns-consumer.ts` — `import * as m from "./mod"` (recorded under `*`).
- `default-consumer.ts` — `import def from "./mod"` (recorded under `default`).

So `used` is referenced only through wildcard records, and the default export is
referenced under `default` — both must count in `dead-exports` / `exports-of`.
