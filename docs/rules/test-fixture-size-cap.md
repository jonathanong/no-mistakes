# `test-fixture-size-cap`

Caps literal fixture cardinalities in configured test and helper files. A
generated row list or a numeric `it.each` table past the configured threshold
is reported. Record tables, dynamic lengths, and files outside the configured
scopes stay silent.

The rule is opt-in. With no `threshold`, it reports nothing. It does not cap
source-file bytes. Every numeric literal in a list passed to a configured
`eachCalls` callee is checked. Use `exemptions`, or a narrower `eachCalls`
list, to leave intended sizes alone.

```yaml
rules:
  - rule: test-fixture-size-cap
    scope: repository
    options:
      threshold: 1000
      exemptions:
        - 1024
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
  property is a numeric literal. An identifier key, a string key `"length"`,
  and a computed string `"length"` all count. A later `length` overrides an
  earlier one.
- `new Array(N)` and a call `Array(N)` when there is exactly one argument and
  that argument is a numeric literal. Both use the `new Array length` message.
  `new Array(1, 2, 3)` and `Array(1, 2, 3)` are element lists, not lengths.
  The callee must be the identifier `Array`. `window.Array` is not matched,
  and the rule does not resolve bindings.
- Calls named by `eachCalls`, default `it.each`, when the first argument is an
  array. Each numeric literal element is reported on its own. Records, holes,
  spreads, nested arrays, and other non-literals are skipped, and literal
  siblings in the same list are still reported.

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

`eachCalls` defaults to `["it.each"]`. Each entry is an `object.property`
identifier pair. An explicit `[]` disables the each form and still checks
`Array.from`, `Array(N)`, and `new Array`. `test.each` is not checked unless it
is listed. Narrow `eachCalls` when a callee's numeric lists should stay
unchecked.

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
// src/orders.ts
export const rows = Array.from({ length: 5_000 }, () => ({ name: "order" }));
```

## Counterexample

```ts
// widgets.test.ts
const rows = Array.from({ length: 1_001 }, () => ({ name: "widget" }));
const wide = Array(10_000);
it.each([10_000, "widget", 100_000])("scale %i", () => {});
```

## Fix

Lower the literal to the threshold or build the rows from a small explicit
list. Add an exact integer to `exemptions`, or narrow `eachCalls`, when a
listed cardinality is intentional. A record element is skipped; numeric
siblings in the same list are still checked.

## Suppression

Use a line or file directive when a literal size is intentional:

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
