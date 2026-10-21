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

const tables = {};
for (const row of Object.values(tables)) {
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

A conditional escape counts: `if (cond) sink(box); box.a = 1` is still a write
after the object may have escaped.

`p || b`, `p ?? b`, and `c ? p : b` keep both sides. Binding that join and
passing the name out, including inside an array or object, releases `b`, so a
later `b.a = 1` is reported. `c && {}` does not: the left side is the result
only when it is falsy, and a falsy value is not the mutated object.

## Allowed

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

The local object is created in that function and the writes finish before it
is returned, passed to a call, stored on a shared object, or captured by a
closure that itself escapes. Calling that closure does not escape the builder
unless the call passes the object out. `Map` and `Set` are allowed. A value
read back from a module-level map, from `Object.values`, or from `.get` is
shared. Class `this` writes are allowed unless `allowThis` is `false`.
`this` in a plain function is reported. `this` in an object method is reported
when that object escapes, including a module-level object, and allowed while
the object stays a local builder. A class that stays local is allowed, including
`new` and a method call. Returning the class, `new` of it, or `fn.bind(...)`
is an escape. A namespace's exported bindings are shared; a const that stays
inside the namespace is still a local builder.

## Options

| Option       | Default | Effect                                                                                                                                                                                                                                                                    |
| ------------ | ------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `module`     | `true`  | Member writes and mutating calls on objects bound at module scope.                                                                                                                                                                                                        |
| `imported`   | `true`  | The same for imported bindings and their members.                                                                                                                                                                                                                         |
| `parameters` | `true`  | The same for parameters and values reached from them.                                                                                                                                                                                                                     |
| `calls`      | `true`  | `Object.assign`, `Object.defineProperty`, `Object.defineProperties`, `Object.setPrototypeOf`, `Reflect.set`, `Reflect.defineProperty`, the same methods via `.call` / `.apply` / `.bind`, and `push`, `pop`, `shift`, `unshift`, `splice`, `sort`, `reverse`, `fill`, `copyWithin`. |
| `allowThis`  | `true`  | Allow `this` writes inside a class body. Plain-function `this` stays reported.                                                                                                                                                                                            |
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

`xs.reduce((acc, x) => { acc[x] = 1; return acc }, {})` is reported because
`acc` is a parameter. `indexed` builds the object in one expression instead.

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
