# `no-mistakes/no-argv-main-guard`

<!-- cspell:ignore realpath -->

## Why

Node can resolve the main module through a symlink while `process.argv[1]` keeps
the invocation spelling. A comparison with the current module's path can then
silently skip the entrypoint. Prefer the runtime's main-module identity.

## Disallowed

```ts
import { fileURLToPath } from 'node:url';
if (process.argv[1] === fileURLToPath(import.meta.url)) main();
const invoked = process.argv.at(1);
if (invoked !== import.meta.filename) return;
if (import.meta.url === `file://${process.argv[1]}`) main();
```

## Allowed

```ts
if (import.meta.main) main();
export function start(isMain = import.meta.main) {
  if (isMain) main();
}
const argument = process.argv[2];
```

## Options

The rule is opt-in and has optional `message` for project-specific guidance.
Use ordinary ESLint `files` or oxlint override configuration for path scoping.

```js
{
  files: ['scripts/**/*.{mjs,mts}'],
  rules: {
    'no-mistakes/no-argv-main-guard': ['error', {
      message: 'Inject isMain or use import.meta.main for executable modules'
    }]
  }
}
```

The rule matches `===`, `!==`, `==`, and `!=` in either operand order, only when
one side is argv index 1 and the other is `import.meta.url`,
`import.meta.filename`, or global CommonJS `__filename`. Optional
`process.argv?.[1]` and `process.argv.at(1)` forms are recognized. Ordinary argv
reads, index 2, `slice(2)`, and comparisons to unrelated paths are not findings.
An exact template with a decoded `file://` prefix, one argv-index-1 substitution,
and no suffix is also recognized, including direct `const` aliases and the
transparent expressions below. Other prefixes, suffixes, multiple substitutions,
and tagged templates are not classified as argv identities.

The global `process` and default/namespace imports or direct `const require`
bindings from `node:process` / `process` are recognized by binding identity.
Direct `const` aliases are followed only for simple identifiers, without writes,
with the initializer text preceding the read. No destructuring projection,
conditional alias, mutation analysis, or inter-file alias search is performed.
Classification follows at most 32 nested expression/alias steps; exhaustion is
unknown and produces no finding. Binding-write checks are memoized per file.

Global `Boolean` is transparent. Single-argument `fileURLToPath`,
`pathToFileURL` (including direct `.href`), `resolve`, and `realpathSync` wrappers
are recognized only from their corresponding Node `url`, `path`, and `fs`
imports or CommonJS bindings. Same-spelled local wrappers are not assumed
transparent. Transparent TypeScript assertions and optional chains are supported.

A named `argv` import, `const { argv }` from `process`, a string or expression-free template index, `.at` with a static `1`, and `fileURLToPath.call` or a literal-array `.apply` are the same access.

## Fix

Use `import.meta.main` on a runtime that supports it, or inject an explicit
`isMain` decision from an entrypoint with a supported runtime-specific mechanism.
CommonJS and older Node runtimes need their own supported entrypoint check; do
not copy ESM syntax into them blindly. The rule has no automatic fix and does
not execute filesystem or symlink probes.

## Limits

Another file's re-export is not visible here. `import { toPath } from "./barrel.mjs"` stays unreported when `barrel.mjs` is `export { fileURLToPath as toPath } from "node:url"` or `export * from "node:url"`. A parameter default such as `function run({ argv } = process)` is not judged, because the caller can pass a different object.

## Suppression

```ts
// no-mistakes-disable-next-line no-argv-main-guard -- intentionally test a legacy entrypoint
if (process.argv[1] === import.meta.filename) main();
```

Line and top-of-file forms (`no-mistakes-disable-line`,
`no-mistakes-disable-file`) are also supported. Directive-looking string literals
cannot suppress a finding. Standard ESLint suppressions also work in ESLint.

## Related rules

- [`async-call-disposition`](async-call-disposition.md) classifies imported calls
  through the same scope-aware binding machinery.
- [`ts-no-const-aliases`](ts-no-const-aliases.md) prevents aliases that obscure
  direct source relationships.
