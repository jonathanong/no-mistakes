# `test-fixture-size-cap`

Caps literal fixture cardinalities in configured test and helper files. A
generated row list past the configured threshold is reported. Record tables,
dynamic lengths, and files outside the configured scopes stay silent. The each
form stays off until `eachCalls` names a callee.

The rule is opt-in. With no `threshold`, it reports nothing. It does not cap
source-file bytes. When `eachCalls` names a callee, the each form reads numeric
elements as cardinalities. Use `eachTableMode: all-elements` when the same
callee runs scale tests and mixed boundary tests. `exemptions` allow exact
cardinalities.

```yaml
rules:
  - rule: test-fixture-size-cap
    scope: repository
    options:
      threshold: 1000
      exemptions:
        - 1024
      eachCalls:
        - it.each
      eachTableMode: all-elements
      scopes:
        tests:
          - "*.test.ts"
          - "**/*.test.ts"
        helpers:
          - "helpers/*.ts"
          - "helpers/**/*.ts"
```

## Why and when

Enable this rule when tests or shared fixture helpers build literal arrays
large enough to dominate runtime or memory. The threshold and the file globs
are configuration, so the check follows the repository's own test layout.

## What it catches

In files matching `scopes.tests` or `scopes.helpers`, the rule reports each
literal cardinality strictly greater than `threshold`:

- `Array.from({ length: N }, ...)` when the object initializer's last `length`
  data property is a numeric literal. An identifier key, a string key `"length"`,
  and a computed string `"length"` all count. A later `length` data property
  overrides an earlier one. A later getter or setter named `length`, a spread,
  or a computed key that is not a known non-`length` literal clears that
  length, so the call stays silent. A computed string or number that is not
  `length` leaves the earlier literal in place.
- `new Array(N)` and a call `Array(N)` when there is exactly one argument and
  that argument is a numeric literal. Both use the `new Array length` message.
  `new Array(1, 2, 3)` and `Array(1, 2, 3)` are element lists, not lengths.
  The callee must be the identifier `Array`. `window.Array` is not matched,
  and the rule does not resolve bindings.
- Calls named by `eachCalls` when the first argument is a literal array.
  `eachTableMode` controls which table elements count and where findings appear.

`1_001` and `0x400` use their integer values. `1_024` and `0x400` are the same
cardinality. A value equal to `threshold` is allowed. An entry in `exemptions`
is allowed even when it is over the threshold.

These forms are silent: a non-literal length (`rows.length`, `count`), a
negative or fractional number, a non-finite number, optional chaining,
a parenthesized callee, a parse error, and a non-JavaScript file. Production
files that match neither scope are not scanned.

## Options

`threshold` has no default. Omit it and the rule reports nothing. Comparison
is `value > threshold`.

`exemptions` defaults to `[]`. Each entry is an exact integer cardinality, not
source text.

`eachCalls` defaults to `[]`. Each entry is an `object.property` identifier
pair. When a callee is listed, numeric elements of its tables are cardinalities
for scale or size tables. `Array.from`, `Array(N)`, and `new Array` are checked
whether or not `eachCalls` is set. `it.each` and `test.each` are not checked
unless they are listed.

`eachTableMode` controls findings for configured `eachCalls`.

| Mode | Behavior |
| --- | --- |
| `per-element` (default) | Reports each oversized numeric literal at the top level of the table. Records, holes, spreads, nested arrays, and other non-literals are skipped. |
| `all-elements` | Reports once on the opening bracket when there is at least one numeric cell and every numeric cell is a non-exempt integer cardinality above the threshold. Reads flat cells and cells in literal row arrays one level deep. Skips boolean and null literals and holes. String labels do not count as cells. The rule skips a duration in seconds matching a `daily` or `every <number><unit>` label, epoch-second values in rows labeled as periods, and compact dates alongside a `YYYY-MM-DD` string. Other numeric cells count normally. An unresolved cell, spread, record, deeper array, or invalid cardinality also keeps the table silent. |

`scopes.tests` and `scopes.helpers` default to `[]`. Empty scopes scan
nothing. A file is in scope when its repository-relative path matches either
glob. Helpers are scanned the same way as tests. `include` and `exclude`
still apply.

## Valid example

```ts
// widgets.test.ts
const boundary = Array.from({ length: 1_000 }, () => ({ name: "widget" }));
const width = Array.from({ length: 1_024 }, () => ({ name: "widget" }));
const listed = new Array(1, 2, 3);
it.each([{ port: 5432 }])("record %j", () => {});
it.each([12, 1_600])("mixed size %i", () => {});
it.each([[48], [1_800]])("mixed row %i", () => {});
it.each([[86_400, "daily"], [7_200, "every 2h"]])("interval %i", () => {});
// src/orders.ts
export const rows = Array.from({ length: 5_000 }, () => ({ name: "order" }));
```

## Counterexample

```ts
// widgets.test.ts
const rows = Array.from({ length: 1_001 }, () => ({ name: "widget" }));
const wide = Array(10_000);
// The table produces one finding with all-elements.
it.each([[1_200], [1_400]])("scale %i", () => {});
```

## Fix

Lower the literal to the threshold or build the rows from a small explicit
list. Add an exact integer to `exemptions` when a listed cardinality is
intentional. Use `all-elements` when a callee named in `eachCalls` also
runs mixed boundary tests.

## Suppression

Use a line or file directive when a literal size is intentional. For
`all-elements`, use the line with the opening bracket of the table.

```ts
// no-mistakes-disable-next-line test-fixture-size-cap
const rows = Array.from({ length: 1_001 }, () => ({ name: "widget" }));
const wide = new Array(8_000); // no-mistakes-disable-line test-fixture-size-cap
```

```ts
// no-mistakes-disable-file test-fixture-size-cap
const rows = Array.from({ length: 1_001 }, () => ({ name: "widget" }));
```

## Related rules

[`test-no-dependency-pins`](test-no-dependency-pins.md) rejects exact version
assertions in tests. [`rust-max-lines-per-file`](rust-max-lines-per-file.md)
caps Rust source length. This rule does not cap file bytes.
