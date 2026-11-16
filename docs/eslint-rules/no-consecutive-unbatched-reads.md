# no-consecutive-unbatched-reads

## Why

Adjacent independent awaited reads add round trips. Configure the client's read contract explicitly and batch the complete key set. This opt-in rule has no client or method defaults and is excluded from presets.

## Disallowed

```ts
import { cache as client } from '@app/cache';
const first = await client.get('a');
const second = await client.ttl('b');
const third = Number(await client.ttl('c'));
const fourth = JSON.parse((await client.get('d')) ?? '{}');
```

## Allowed

```ts
import { cache as client } from '@app/cache';
const first = await client.get('a');
const second = await client.get(first.nextKey);
const values = await Promise.all([client.get('a'), client.ttl('b')]);
```

## Options

The options object accepts `clients` (exact local identifier names), `imports` (runtime named-import selectors with exact `source` and `imported` export names), required `readMethods` (exact method names), and `message` (custom diagnostic text). Import aliases are resolved by binding; type imports and shadowed import names do not match. A `clients` name explicitly asserts that contract for every binding with that local name, including parameters.

```js
{ 'no-mistakes/no-consecutive-unbatched-reads': ['error', {
  imports: [{ source: '@app/cache', imported: 'cache' }],
  readMethods: ['get', 'ttl'],
}] }
```

Adjacent reads in a module, block, or individual switch case are compared. Supported statement shapes are expression statements, simple assignments, variable declarators (including exported declarations), and a second read written as a return statement. The awaited read may occur inside call arguments, conversions, member access on its result, parentheses, or transparent TypeScript assertions. A read on the left side of `&&`, `||`, or `??`, or in a conditional expression's test, is unconditional and remains eligible.

Exactly one configured awaited read must occur outside function boundaries in the expression, and it must be unconditional. A statement with multiple configured awaited reads is ambiguous and breaks adjacency, including when one read is conditional or nested in another read's arguments. Reads inside callbacks, arrows, or nested functions do not belong to the containing statement. Reads on the right side of a logical expression, in either conditional branch, in optional-call arguments or optional computed access, or through an optional client call are not unconditional and are not paired. Reading an optional property of an already awaited result does not make the read itself conditional.

Declarators are compared in execution order, including across statement boundaries; a non-read declarator breaks adjacency. Separate switch cases are never combined, and a return ends a sequence. Static computed method names are accepted. Both reads must target the same resolved client binding. Referencing any previous declared or assigned result binding, including a destructured binding, anywhere in the second read call exempts the pair. A dependency only in the surrounding conversion does not change whether that read can be batched. Compound assignments, assignments to properties or unresolved bindings, aliases, arbitrary control flow, and helper return values are not followed. Configured methods assert independent reads when no result dependency is visible; the rule does not prove purity of argument helpers or discover hidden dependencies. Keep such dependencies explicit in the second call or separate the statements.

## Fix

Use the client's batch API or a suitable concurrent operation when ordering and failure semantics permit. No automatic fix changes execution order.

## Suppression

```ts
await client.get('a');
// no-mistakes-disable-next-line no-consecutive-unbatched-reads -- ordering required
await client.ttl('b');
```

For multiple declarators, a directive on the declaration header covers the declaration; a directive on an individual declarator covers that read. Also supports `no-mistakes-disable-line`, a leading `no-mistakes-disable-file`, and ordinary ESLint/Oxlint suppression directives.

## Related rules

- [require-top-level-registration](require-top-level-registration.md)
