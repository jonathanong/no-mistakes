# `no-mistakes/no-object-mutation`

Opt-in. Enabling this rule does not change the analyzer. A file that skips the
rule can still mutate, and a write the analyzer cannot follow stays Unknown.

## Why

A write through a module binding, an import, or a parameter makes that
object's contents depend on call order. Value-flow and embedded SQL then have
to guess, and a miss becomes an unknown value or unanalyzable SQL. Build a new
value instead of teaching the analyzer another mutation shape.

`delete` is [`no-delete-property`](no-delete-property.md). This rule does not
report it.

ESLint and Oxlint `no-param-reassign` with `{ props: true }` only see a direct
write on the parameter. An alias, a module binding, an import, and
`Object.assign` / `push` are out of scope. `eslint-plugin-functional`'s
`immutable-data` flags essentially every write, including a local builder after
`const box = {}`, and its allow lists are regular expressions. Oxlint has no
equivalent. None of those configs is a substitute.

## Disallowed

```ts
export const tables = {};
export function register(name, table) {
  tables[name] = table;
}

import { rows } from "./rows";
rows.push(row);

function write(box) {
  box.client.get = replica.get;
}

function touch(box, rows) {
  arguments[0].b = 1;
  const alias = arguments;
  alias[0].c = 1;
  Reflect.set(box, "a", 1);
  Reflect.defineProperty(box, "b", { value: 1 });
  Array.prototype.push.call(rows, 1);
  [].push.apply(rows, [2]);
}

const cache = new Map();
cache.get(1).ready = true;

const tableRows = {};
for (const row of Object.values(tableRows)) {
  row.ready = true;
}

namespace Registry {
  export const rows = {};
}
Registry.rows.users = "app.users";

function publish() {
  this.client = null;
}

const api = {
  save() {
    this.dirty = true;
  },
};
```

Same-file method aliases retain the receiver they were read from. Static string
and expression-free template keys, optional calls, and a direct destructure without a
default are included. A forwarded call uses its actual `this` receiver:

```ts
function append(rows, otherRows, row) {
  const push = rows["push"];
  const alias = push;
  alias?.(row);
  const { [`push`]: appendRow } = rows;
  appendRow(row);
  alias.call(otherRows, row);
  alias.apply(otherRows, [row]);
  const bound = alias.bind(otherRows);
  bound(row);
  bound.call();
}
```

A local array that stages caller-owned values does not make them local. Reading
a staged entry back, including through a destructure or a `for...of`, keeps the
provenance of what was pushed. `entries()` and `values()` on a caller-owned
collection yield caller-owned elements, so a staged `body` and `event` are both
reported:

```ts
export async function stage(bodies: unknown[], events: Ev[]) {
  const pending: Array<{ args: Record<string, unknown>; event: Ev }> = [];
  for (const [i, body] of bodies.entries()) {
    pending.push({ args: body as Record<string, unknown>, event: events[i]! });
  }
  for (const { args, event } of pending) {
    args["id"] = 1; // reported: `body` is the caller's
    event.target = { id: "x" }; // reported: `events[i]` is the caller's
  }
}
```

That caller-owned yield applies to an untyped receiver, to `any` and `unknown`,
to unions and type parameters, and when `entries` or `values` comes from
TypeScript's default library or an external library (`Array`, a tuple,
`readonly` arrays, `Map`, `Set`, and subclasses that inherit those methods).
It also applies to a project-declared `values()` or `entries()` unless its body
provably returns a fresh collection. Every `return` must be an array literal
(including spreads), `new Map(...)`, `new Set(...)`, `new Array(...)`,
`Array.from(...)`/`Array.of(...)`, or a `.map`, `.filter`, `.slice`, `.concat`,
`.flat`, `.flatMap` or `.toSorted` call. A returned `this` member, parameter,
module-level binding, interface or abstract signature, generator, or anything
else unproven stays caller-owned, so the write is reported:

```ts
class Registry {
  private byId = new Map<string, Row>();
  values() {
    return this.byId.values();
  }
}
const registry = new Registry();
for (const row of registry.values()) {
  row.x = 1; // reported: the iterator walks the registry's shared map
}

class Snapshot {
  private rows: Row[] = [];
  values() {
    return [...this.rows];
  }
}
const snapshot = new Snapshot();
for (const row of snapshot.values()) {
  row.x = 1; // quiet: `values` returns a fresh array
}
```

Without checker services, including the Oxlint plugin, that call stays
caller-owned. A union stays caller-owned when any member is. A type parameter
uses its constraint, and a parameter with no constraint stays caller-owned.

Fresh entries staged in one local array share one provenance union. Escaping
any read of that array escapes every fresh entry in the union, so a later write
through a different index still reports:

```ts
type Ev = { name: string; target?: { id: string } };
function leak(value: unknown) {
  return value;
}
export function tainted() {
  const pending: Array<{ event: Ev }> = [];
  pending.push({ event: { name: "one" } });
  pending.push({ event: { name: "two" } });
  leak(pending[0]);
  pending[1]!.event.target = { id: "x" }; // reported: the entries share one union
}
```

Bare `this.push(...)`, `this.pop()`, and the other listed array mutator calls
are not presumed to mutate an array when the receiver is unknown. This includes
stream methods in object literals and ordinary functions. Known array receivers
and explicit array-method forwarding such as `Array.prototype.push.call(this, x)`
retain normal reporting. Calls on members such as `this.items.push(x)` and
`Object.assign(this, ...)` also retain normal reporting.

A conditional escape counts: `if (cond) sink(box); box.a = 1` is still a write
after the object may have escaped.

`p || b`, `p ?? b`, and `c ? p : b` keep both sides. Binding that join and
passing the name out, including inside an array or object, releases `b`, so a
later `b.a = 1` is reported. `c && {}` does not: the left side is the result
only when it is falsy, and a falsy value is not the mutated object.

## Allowed

An inline synchronous `reduce` or `reduceRight` callback may build its first
parameter when the initial value is an object or array literal and every
callback return is that parameter. Only direct container writes and array mutator
calls on that parameter are exempt; the block must end with `return acc`.
TypeScript assertions around the literal are supported:

```ts
const byId = rows.reduce((acc, row) => {
  acc[row.id] = row;
  return acc;
}, {} as Record<string, Row>);

const reversed = rows.reduceRight((acc, row) => {
  acc.push(row);
  return acc;
}, []);
```

Identifier seeds, omitted seeds, async callbacks, accumulator rebinding, aliases,
member reads, nested writes, and callbacks returning another value retain ordinary
parameter reporting. Build a complete nested value locally, then assign it through
a direct root write such as `acc[key] = value`. Other callback parameters and
shared values nested inside a literal remain shared.
Passing the accumulator to an unknown call or publishing a closure that captures
it escapes it; subsequent writes, including writes in later iterations, are
reported. Publishing a nested member also escapes that member. The result keeps
this provenance after the reduction. Construct the complete local value before
publishing it, and avoid mutating a member after publishing that member.
The exemption assumes the native array callback contract for unknown receivers.
Known local custom object or class methods named `reduce` keep ordinary callback
parameter reporting.

```ts
function build(extra) {
  const box = {};
  box.client = { get: extra };
  return box;
}

const cache = new Map();
cache.set(id, value);

class Client {
  setHost(host) {
    this.host = host;
  }
}

function localClass(value) {
  const box = {};
  class Writer {
    set() {
      box.current = value;
    }
  }
  new Writer().set();
  return box;
}
```

```ts
function buildRows(row) {
  const rows = [];
  const push = rows.push;
  push(row);
  return rows;
}

function forwardToLocal(sharedRows, row) {
  const push = sharedRows.push;
  const local = [];
  push.call(local, row);
  const bound = push.bind(local);
  bound(row);
  return local;
}
```

An early return of a fresh local accumulator does not make later writes to the
remaining path shared. The same holds when the return is inside a block or an
`else` branch, and when a fresh nested value is built after the early return:

```ts
function build(skip) {
  const rows = [];
  if (skip) return rows;
  rows.push("local");
  return rows;
}

function buildNested(skip) {
  if (skip) return { rows: [] };
  const result = { rows: [] };
  const rows = result.rows;
  rows.push("local");
  return result;
}
```

Writes after a real escape still report, including when the escape occurs on
the path after the early return. Returning an escaping bound mutator or closure
also exposes its captured values. Returning `array.push(...)`'s numeric result
does not expose the array itself. Callbacks reachable on a return path remain
checked even if the continuing path replaces a callback field or captured binding.

An inline synchronous callback passed to a known array's built-in `map`,
`forEach`, `flatMap`, or `filter` runs before the call returns. It may mutate a
fresh local builder captured from the enclosing function while that builder
remains locally owned:

```ts
function collect() {
  const out = [];
  [1, 2].forEach((value) => out.push(value));
  return out;
}
```

This preserves ownership; it does not exempt all writes in the callback.
Parameters, imports, module bindings, and shared values nested inside a fresh
container retain ordinary reporting. Passing the builder to an unknown call or
publishing a closure that captures it escapes it. Subsequent writes still report,
including a write before that escape in source order when an earlier callback
iteration may already have escaped the builder. An early callback return does
not erase an escape that another iteration can observe.

Known receivers include direct array literals, local bindings holding fresh
array values, and parameters directly annotated with an array type (`T[]`),
a tuple type, or the global `Array<T>` type. Unmodified global
`Object.entries`, `Object.keys`, `Object.values`, and `Array.from` also produce
known arrays, whether used directly or through local bindings. An `as` or
`satisfies` wrapper around a proven array preserves that proof:

```ts
function collect(groups: Record<string, number>) {
  const entries = Object.entries(groups) as Array<[string, number]>;
  const ready: number[] = [];
  entries.forEach(([, value]) => ready.push(value));
  return ready;
}
```

With typed ESLint (`parserOptions.project` or `projectService`), the checker can
also prove array and tuple aliases, member receivers, typed call results, and
unions whose every member is an array or tuple. For these newly proven
receivers, the iteration method must come only from TypeScript's default
library. Custom declarations, `any`, `unknown`,
and non-array types do not establish this proof. Assertions cannot manufacture
array provenance from an unknown value, including through local aliases.
Without checker services, including in the Oxlint plugin, those typed-only
receivers remain unproven.

Shadowed constructors, modified global factories, and escaped constructor
references do not prove array producers. The additional proof checks aliases
and indirect escapes before admitting a newly recognized receiver.
Array recognition preserves the existing ownership model. `Object.values`
retains its prior shared-value handling; recognizing `Object.entries` or
`Array.from` does not infer new element ownership. Mapper results remain opaque,
and mapper callbacks retain ordinary escape handling. Fresh literal results
such as `Array.from({ length: count }, () => ({}))` remain clean. Staging a
parameter-array element in a local pending array also retains its prior
diagnostics; its borrowed-child provenance gap is tracked separately in
[#3646](https://github.com/jonathanong/no-mistakes/issues/3646). Receiver alias
and indirect-escape checks apply to the newly
recognized iterations without changing existing array diagnostics. Only inline,
synchronous, non-generator iteration callbacks receive ownership preservation;
named
callbacks and async callbacks retain ordinary handling.
Callbacks that write object properties, use `Object` or `Reflect` mutation,
or read nested members of captured fresh containers also retain ordinary
handling. Those operations can carry shared children between iterations;
the synchronous path supports direct array mutator calls on captured builders.

Any tracked member write or mutation through `Object` or `Reflect` invalidates
built-in method recognition for that receiver, even when the write names another
property. Passing an annotated receiver to an unknown call also invalidates
recognition. A fresh receiver that has escaped falls back to ordinary call
handling, as does an explicit write to the global array prototype. Custom or
overridden methods do not receive ownership preservation.
Callbacks passed to unresolved calls may escape; returning a captured outer
builder does not restore its local ownership. Build a fresh value inside each
callback instead.

Return-path handling restores fresh-value ownership for continuing paths. It
retains nested callback escape effects and the escape epoch used to revisit
loops: restoring those would discard evidence from the terminating path. A
temporary return escape can therefore request another loop pass without making
a continuing-path builder shared.

Computed member receivers keep the ownership of their possible children. A local
literal containing fresh arrays can be filled through either a computed or static
key before it escapes:

```ts
export function group(items: { kind: "a" | "b"; v: number }[]) {
  const out: Record<"a" | "b", number[]> = { a: [], b: [] };
  for (const item of items) out[item.kind].push(item.v);
  return out;
}
```

A computed read of a parameter, import, or module binding stays shared. A shared
or escaped child inside a local literal also stays shared through a computed
read; selecting a child does not create a fresh copy.

The local object is created in that function and the writes finish before it
is returned, passed to a call, stored on a shared object, or captured by a
closure that itself escapes. Calling that closure does not escape the builder
unless the call passes the object out. `Map` and `Set` are allowed. A value
read back from a module-level map, from `Object.values`, or from `.get` is
shared. Class `this` writes and functions with a declared receiver are allowed unless
`allowThis` is `false`. Direct `this.field = value` initialization in a
constructor is allowed even with `allowThis: false` when that class declares the
instance field, including a private `#field`. Public and private names are distinct.
Inherited, undeclared, and static fields, methods, setters,
compound writes, nested functions and arrows, mutator calls, and
`Object.assign(this, ...)` do not receive this initialization exemption.
For example, this supports constructor arguments with native type stripping:

```ts
class Failure extends Error {
  readonly reason: unknown;
  constructor(reason: unknown) {
    super("failure");
    this.reason = reason;
  }
}
```

Known array receivers include tuple and array `this` annotations, the global
`Array<T>` type, and subclasses of array-derived classes. A locally declared
`Array` type does not establish array provenance. Static class methods do not
inherit array-instance provenance.

A receiver is declared by a TypeScript `this: T` parameter
or an attached JSDoc `@this {T}` tag. Arrows inherit the enclosing receiver; a
nested regular function must declare its own. `this` in a plain function without
a declared receiver is reported. `this` in an undeclared object method is reported
when that object escapes, including a module-level object, and allowed while
the object stays a local builder. A class that stays local is allowed, including
`new` and a method call. Returning the class, `new` of it, or `fn.bind(...)`
is an escape. A namespace's exported bindings are shared; a const that stays
inside the namespace is still a local builder.

A declared receiver also supports object methods supplied to extension APIs:

```ts
interface Context { data?: string }

const extensions = {
  update(this: Context) {
    this.data = "ready";
    (() => { this.data = "done"; })();
  },
};

export function apply(app: { extend(value: object): void }) {
  app.extend(extensions);
}
```

```js
/** @this {{ data?: string }} */
export function update() {
  this.data = "ready";
}
```

Entries that are fresh in this function stay local when staged. A pending array
of events created inside the function, or of literals produced by a mapper, is
still written freely:

```ts
export function stage(names: string[]) {
  const pending = names.map((name) => ({ args: {} as Record<string, unknown>, name }));
  for (const { args } of pending) args["id"] = 1;
}
```

## Options

| Option       | Default | Effect                                                                                                                                                                                                                                                                    |
| ------------ | ------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `module`     | `true`  | Member writes and mutating calls on objects bound at module scope.                                                                                                                                                                                                        |
| `imported`   | `true`  | The same for imported bindings and their members.                                                                                                                                                                                                                         |
| `parameters` | `true`  | The same for parameters and values reached from them.                                                                                                                                                                                                                     |
| `calls`      | `true`  | `Object.assign`, `Object.defineProperty`, `Object.defineProperties`, and `Object.setPrototypeOf`, including those `Object` methods via `.call`, `.apply`, and `.bind`. `Reflect.set` and `Reflect.defineProperty`. `push`, `pop`, `shift`, `unshift`, `splice`, `sort`, `reverse`, `fill`, and `copyWithin`, including those array methods via `.call` and `.apply`. |
| `allowThis`  | `true`  | Allow `this` writes inside a class body, or in functions and methods declaring a TypeScript `this: T` parameter or attached JSDoc `@this {T}`. Arrows inherit that receiver; nested regular functions need their own declaration. With `false`, direct constructor `this.field = value` initialization of an own declared instance field remains allowed; nested callbacks, other writes, and mutator calls retain reporting.                                                                                                                                                                                            |
| `allow`      | `[]`    | Root names or dotted paths. `tables` allows `tables[key]`. `tables.users` allows `tables.users.id` but not a computed `tables[key]`.                                                                                                                                      |

## Fix

Declare the value once. A spread copy is not a known object, so it does not
remove the blind spot.

```ts
export const tables = { users: "app.users", orders: "app.orders" } as const;

export function replicaClient(get) {
  return { client: { get } };
}

query(`SELECT id FROM ${tables.users} WHERE active`);

function listed(row) {
  return [row];
}

function indexed(xs) {
  return Object.fromEntries(xs.map((x) => [x, 1]));
}
```

`xs.reduce((acc, x) => { acc[x] = 1; return acc }, {})` is allowed because
its literal seed is locally owned. With an identifier seed such as `shared`,
`xs.reduce((acc, x) => { acc[x] = 1; return acc }, shared)` is still reported.
Build a fresh literal accumulator instead. `indexed` is another option that
builds the object in one expression.

`literal-rewrite.mts` declares `tables` once, so the query resolves
`app.users`, and `replicaClient` returns a known object. `rows-rewrite.mts`
is the rewrite for `rows.push(row)`: the return is a known array.
`literal-mutated.mts` assigns those fields afterward, invalidates the object,
and leaves the query unanalyzable. `rows-mutated.mts` records `rows.push(row)`
and still returns the import name, not a known array. Those files are under
`test-cases/codebase-analysis/no-object-mutation-literal/fixture/`.

There is no autofix. `Object.assign` onto an existing object is not the same
call as a fresh literal, including across files.

## Limits

A declaration directly in module scope is shared immediately. The same
declaration inside a nested block stays a local builder until the value
escapes, because that block is not the module environment. Writes through
`module.exports`, `exports`, `globalThis`, `window`, and `process.env` are
not flagged.

Aliases of the listed array mutators use the existing receiver provenance and
options. A later reassignment of the receiver name does not change the object
captured by an earlier alias. Reassigning the alias replaces its provenance.
Passing or returning an alias escapes its retained receiver; an unknown caller's
later invocation is not analyzed. `.call`, literal-array `.apply` (without holes
or spreads), and `.bind` use their explicit receiver, rather than the object
from which the method was extracted. Binding alone does not mutate it. A bound alias keeps its first bound receiver
across later `.call`, `.apply`, or `.bind` wrappers.

A computed identifier key, a destructuring default, a rest pattern, a
parameter-default destructure, and another file's re-export do not establish a
mutator alias. For example, none of these calls proves a mutation of `rows`:

```ts
function unknown(rows, key, fallback) {
  const dynamic = rows[key];
  dynamic(1);
  const { push = fallback } = rows;
  push(1);
  const { ...rest } = rows;
  rest.push(1);
}

function defaulted({ push } = rows) {
  push(1);
}

export { push } from "./mutators";
```

A conditional or logical choice between different method values does not
establish one mutator alias. Nested destructuring patterns and method values stored in
object or array fields are also outside this binding-alias tracking.

Static aliases of `Object` and `Reflect` methods are outside this alias tracking;
the directly spelled forms in Options retain their existing behavior.

## Suppression

```ts
// eslint-disable-next-line no-mistakes/no-object-mutation -- third-party cache
external.entries[key] = value;
```

```ts
// no-mistakes-disable-next-line no-object-mutation -- third-party cache
external.entries[key] = value;
```

`no-mistakes-disable-line` and `no-mistakes-disable-file` work the same way.
Do not use `guardrails` directives.

## Related rules

- [`no-delete-property`](no-delete-property.md) reports `delete` on a property.
