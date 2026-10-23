# require-top-level-registration

## Why

Owner modules must install lifecycle callbacks when imported. A registration hidden behind a condition or deferred function leaves cleanup dependent on runtime control flow. This opt-in rule is excluded from presets. Scope it to owner modules with ESLint `files` or Oxlint overrides.

## Disallowed

```ts
import { register } from '@app/lifecycle';
if (enabled) register(onStop);
```

## Allowed

```ts
import { register as attach } from '@app/lifecycle';
attach(onStop);
```

## Options

The options object requires `registrar` with exact `source` and `imported` export names and `callback` (the local callback identifier). Optional `message` replaces the diagnostic text.

```js
{ files: ['src/lifecycle-owner.ts'], rules: {
  'no-mistakes/require-top-level-registration': ['error', {
    registrar: { source: '@app/lifecycle', imported: 'register' },
    callback: 'onStop',
  }],
} }
```

The registrar must resolve to a runtime named import from the configured source. Import aliases and string export names work; type-only imports, unrelated modules, local functions, namespace calls, optional calls, and indirect aliases do not satisfy the contract. Registration must be a direct module-level expression statement, with exactly the configured identifier argument. Conditional expressions, functions, loops, nested blocks, and exported variable initializers do not count. The rule checks static call placement and import identity; it does not prove that earlier module initialization finishes or that the callback has a particular runtime implementation. The callback name is a local binding contract; its declaration need not be in this file.

## Fix

Import the runtime registrar and call it directly at module top level with the configured callback. No automatic fix moves side effects.

## Suppression

The diagnostic is anchored to the first module statement (line 1 in an empty module). Line directives target that location.

```ts
// no-mistakes-disable-next-line require-top-level-registration -- alternate lifecycle owner
import { register } from '@app/lifecycle';
```

Also supports `no-mistakes-disable-line`, a leading `no-mistakes-disable-file`, and ordinary ESLint/Oxlint suppression directives.

## Related rules

- [no-consecutive-unbatched-reads](no-consecutive-unbatched-reads.md)
