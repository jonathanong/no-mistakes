# `queries` fixture

Shared fixture for the lightweight query commands (`importers`, `exports-of`,
`dead-exports`, `call-sites`, `resolve-check`).

- `util.ts` — defines `used` (called from several files), `dead` (never
  imported), and `helper`.
- `consumer.ts` — imports and calls `used`/`helper`, including a spread call and
  a top-level call site (null caller).
- `barrel.ts` — re-exports `used` (references it without calling it).
- `consumer.test.ts` — a test that transitively reaches `util.ts` (for
  `importers --tests`).
- `broken.ts` — mixes a resolving import, a broken relative import, a broken
  `@app/*` alias import, a Node builtin, and a bare npm package.
- `tsconfig.json` — declares the `@app/*` alias used by `broken.ts`.
