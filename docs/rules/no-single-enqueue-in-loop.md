# `no-single-enqueue-in-loop`

Reports a configured single-item enqueue when the prepared per-item proof places
that call inside an open loop or an open `map`, `forEach`, or `flatMap` callback.
Call the configured bulk API once with the collection instead.

## Why and when

One enqueue per element multiplies queue writes by the size of the collection.
A bulk API accepts the collection in one call. The rule is opt-in: with no
configured single targets it reports nothing, and it does not assume any queue
or database API by name.

## What it catches

It reports a direct call, or a bare identifier passed as a callback, whose
resolved import is a configured `options.single` target. The call must already
be in the shared per-item occurrence list, and the construct must be `for`,
`for-of`, `for-in`, `while`, `do`, `map callback`, `forEach callback`, or
`flatMap callback`.

`filter`, `some`, `every`, `reduce`, and `find` callbacks are not reported.
`for await` is reported as `for-of`: that is the shared per-item construct for
both `for-of` and `for await`. A bare callback such as `items.forEach(enqueueItem)`,
a `map` callback nested in `Promise.all`, and a bare `map` callback in a loop
header (`for (const item of items.map(enqueueItem))`) are reported when the
shared proof records that callback. A local wrapper called from the loop, and a
call inside a nested function or arrow, are not followed: those calls have a
different caller and may not run once per item. The rule binds a call only when
the canonical graph resolves that call to an imported module export, so
`deps.enqueueItem` stays out of scope. A configured `options.bulk` target is
never a finding, including when a loop walks batches and calls the bulk API once
per batch. Bulk names the export that already sends the collection. Validation
rejects the same module and export in both lists. A bulk spelling that resolves
to the same file and export as a single target does not skip that call. A
different bulk export still skips. The same local name imported from a different module stays
silent. An alias and a namespace member are judged by the resolved module file
and export name. A namespace member is reported when the canonical graph
resolves it to the configured export.

The rule does not count iterations. It reports when the shared per-item proof
includes the call. A `for` whose counter, bound, and step are closed numbers,
and whose body does not rewrite the counter, is not part of that proof, so
`for (let index = 0; index < 1; index += 1)` stays silent. A bound that is not
that proof is still a finding, including when the argument is not the loop
element: `for (let index = 0; index < limit; index += 1) enqueueItem(order)`
reports. A `const` array binding such as `const copied = [order]` is not a
closed collection, so `for (const item of copied)` reports. A closed literal
tuple (`["widget"] as const`) is the existing closed-collection proof, and a
`for-of` or array callback over that proof stays silent.

## Options and defaults

`options.single` and `options.bulk` default to empty lists. Each entry names an
import specifier and an export. The specifier is the text you would write in an
import, such as `./widgets`, `widget-queue`, or a tsconfig `paths` alias such as
`@widgets`. A call matches when its specifier text is that module and export, or
when the shared resolver produces the same file for the configured specifier and
for the call. A relative import from another directory, a package name, and a
paths alias can name one module. When one relative spelling resolves to more
than one file, each of those files matches. Blank module or export text is an
error. The same module
and export cannot appear twice in one list, or in both lists. `message`, when
set, is prefixed to the finding. `include` and `exclude` are the shared path
filters.

```yaml
rules:
  - rule: no-single-enqueue-in-loop
    options:
      single:
        - module: ./widgets
          export: enqueueItem
      bulk:
        - module: ./widgets
          export: enqueueItems
```

There is no separate low-effort implementation. `effort: low` warns
`requested effort low; effective effort max because this rule has no low implementation`
and runs this rule at `max`.

## Valid example

```ts
// src/widgets.ts
export function enqueueItem(item: string): void {
  void item;
}

export function enqueueItems(items: readonly string[]): void {
  void items;
}

// src/index.ts
import { enqueueItems } from "./widgets";

export async function publish(items: readonly string[]): Promise<void> {
  await enqueueItems(items);
}
```

## Counterexample

```ts
// src/widgets.ts
export function enqueueItem(item: string): void {
  void item;
}

export function enqueueItems(items: readonly string[]): void {
  void items;
}

// src/index.ts
import { enqueueItem } from "./widgets";

export function publish(items: readonly string[]): void {
  for (const item of items) {
    enqueueItem(item);
  }
}
```

## Fix

Call the configured bulk API once with the collection. A loop that already
calls that bulk API, one batch at a time, can stay. A proven one-iteration
numeric `for` does not need a change. Move a true per-item send out of an open
loop, or replace `items.map(enqueueItem)` and `items.forEach(...)` with the
bulk call.

## Suppression

Use `no-mistakes-disable-next-line no-single-enqueue-in-loop` on the enqueue
line, `no-mistakes-disable-line` on that same line, or
`no-mistakes-disable-file` when the whole file is a reviewed exception. Say why
the single call is bounded.

## Related rules

[`query-reached-per-item`](query-reached-per-item.md) follows configured effect
sinks through helpers. This rule does not. It only matches the configured
single and bulk imports at the per-item call itself.

CLI: `no-mistakes check --root . --format json`.
