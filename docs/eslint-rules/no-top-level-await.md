# `no-mistakes/no-top-level-await`

## Why

Keep asynchronous initialization out of module evaluation. Top-level `await`
suspends evaluation; importers through `export *` barrels can observe uninitialized
bindings under Bun or some bundlers. Prefer an exported async initialization
function, called explicitly by the entrypoint.

This file-local rule is opt-in in ESLint and Oxlint. It reports `await` expressions,
`for await` loops, and asynchronous resource disposal (`await using`) declarations
outside functions, including nested blocks, class heritage, and computed class
keys. Function and method bodies are allowed. It does not inspect dependency
graphs.

## Disallowed

```ts
export const settings = await loadSettings();
for await (const event of events) {
  consume(event);
}
await using handle = openHandle();
```

## Allowed

```ts
export async function initialize() {
  return await loadSettings();
}
if (import.meta.main) {
  await initialize();
}
```

## Options

```js
{
  rules: {
    'no-mistakes/no-top-level-await': ['error', {
      allowInImportMetaMain: true,
    }],
  },
}
```

`allowInImportMetaMain` defaults to `true`: awaits within the consequent of a
direct `if (import.meta.main)` are allowed, including nested blocks. Parentheses
and TypeScript assertion wrappers around the condition are accepted. The test,
`else` branch, negated tests, computed properties and compound tests do not earn
an exemption. Set this option to `false` to report guarded awaits too.

`message` replaces the diagnostic with project-specific guidance. Use flat-config
`files` and `ignores` to exempt dedicated scripts or entrypoints:

```js
{
  files: ['src/**/*.{ts,mts}'],
  ignores: ['src/entrypoint.mts'],
  rules: { 'no-mistakes/no-top-level-await': 'error' },
}
```

## Fix

Move initialization into an exported async function and await it from an
entrypoint. There is no automatic fix: wrapping code can change export and
initialization semantics.

## Suppression

Use a narrow directive with a reason:

```ts
// no-mistakes-disable-next-line no-top-level-await -- dedicated runtime entrypoint
await initialize();
```

`no-mistakes-disable-line` and a leading `no-mistakes-disable-file` also work,
with either the short rule name or `no-mistakes/no-top-level-await`.

## Related rules

- [`no-argv-main-guard`](no-argv-main-guard.md) checks entrypoint identity comparisons.
- [`async-call-disposition`](async-call-disposition.md) checks how promises are consumed.
