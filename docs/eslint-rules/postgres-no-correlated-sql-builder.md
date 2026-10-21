# `no-mistakes/postgres-no-correlated-sql-builder`

## Why

Two or more guards that append to the same builder for the same selector do
not describe one statement. An unknown selector can take none of those guards
and leave the builder empty, or take a path the file never wrote.

Return a complete statement from one switch with a throwing default. That
shape is judged: `accessSql` in
`test-cases/rules/postgres-required-predicates/variants/proven-throw-switch/src/queries.mts`
returns a full `sql` query from each arm, and `byAccess` is checked per arm.
A default that does not throw, including a function named `assertNever`,
publishes nothing.

This rule is opt-in. It is not part of `configs.recommended` or `configs.strict`.
It does not autofix. Ordinary object mutation stays in
[issue #2219](https://github.com/jonathanong/no-mistakes/issues/2219).

## Disallowed

```ts
import sql from "sql-template-strings";

export function byAccess(query: { append(text: unknown): unknown }, access: string) {
  if (access === "owned") query.append(sql`AND account_id = $1`);
  if (access === "member") query.append(sql`AND member_id = $1`);
  return query;
}
```

## Allowed

```ts
import sql from "sql-template-strings";
import { read } from "@example/db";

export function accessSql(access: string) {
  switch (access) {
    case "owned":
      return sql`SELECT id FROM orders WHERE account_id = $1`;
    case "member":
      return sql`SELECT id FROM orders WHERE member_id = $1`;
    default:
      throw new Error("unknown access");
  }
}

export function byAccess(access: string) {
  return read(accessSql(access), [1]);
}
```

One guard, two selectors, two builders, or an append that is not a string,
template, or configured tag is not this pattern.

## Options

The rule reports nothing until both lists are non-empty. `builderMethods` is
the method names that append SQL, such as `["append"]`. `tagNames` is the tag
identifiers that produce SQL text, such as `["sql"]`. A string or untagged
template passed to one of those methods counts as SQL text too.

## Fix

This rule has no autofix and no editor suggestion. Replace the guards with a
switch that returns a complete statement from every arm and throws in
`default`. Do not suggest an own-key check; that form is not judged.

## Suppression

```ts
// no-mistakes-disable-next-line postgres-no-correlated-sql-builder -- reviewed builder
if (access === "owned") query.append(sql`AND account_id = $1`);
```

`no-mistakes-disable-line` and `no-mistakes-disable-file` work the same way.

## Related rules

[`postgres-no-nonliteral-sql-registry-key`](postgres-no-nonliteral-sql-registry-key.md)
covers a runtime key into a const identifier registry.
[`postgres-no-async-sql-builder`](postgres-no-async-sql-builder.md) covers an
async builder that never waits.
