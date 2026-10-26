# `no-mistakes/postgres-no-correlated-sql-builder`

## Why

Independent guards that append to the same builder for the same selector can
leave the statement incomplete. An unknown selector can take none of those guards
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

The rule also abstains on a complete `if` / `else if` chain whose final `else`
throws, and on direct guards that each return followed by an unconditional
throw at the end of the function. These shapes exclude an unknown selector
without publishing an empty builder. Completion is checked only through direct
statements and straight-line blocks; a conditional throw or an earlier return
inside the final `else` does not prove completion. Every arm must execute its
recognized append unconditionally. A chain under another condition or a loop
is still reported because that outer construct can bypass the whole chain.

```ts
if (access === "owned") query.append(sql`AND account_id = $1`);
else if (access === "member") query.append(sql`AND member_id = $1`);
else throw new Error("unknown access");
```

## Options

```js
{
  "no-mistakes/postgres-no-correlated-sql-builder": [
    "error",
    { builderMethods: ["append"], tagNames: ["sql"] },
  ],
}
```

The rule reports nothing until both lists are non-empty. `builderMethods` is
the method names that append SQL, such as `["append"]`. A string literal or
expression-free template is that method (`query["append"]` and
``query[`append`]``). A computed identifier key is not. `query.append.call(query, text)` forwards to `append`. The SQL text is the
second argument, and the builder is the receiver in the first argument. A
receiver that is not an identifier, a spread argument, or a different object
on each guard does not count as one builder. A configured method whose name
is `call` or `apply` stays that method. Literal-array forwarding
`query.append.apply(query, [text])` uses the same explicit receiver. Immutable
same-file method aliases (`const append = query.append; append.call(query,
text)`) and comma method references (`(0, query.append).call(query, text)`)
retain that receiver; optional calls and TypeScript assertions are transparent.
A tag that is not a member or identifier, such as `` factory()`...` ``, stays silent.

Forwarding requires one SQL piece and an identifier receiver. Unknown argument
arrays, holes, spreads, extra arguments, reassigned or `let`/`var` aliases,
computed identifier method keys, and unresolved or shadowed method bindings stay
unknown. A detached invocation (`const append = query.append; append(text)`)
does not prove a builder receiver. Rewrite these to a direct configured append
with an explicit builder; replace repeated guards with the complete statement
`switch` and throwing default described below. Guards that forward to different
receivers do not establish one correlated builder. Each guard retains every
distinct builder it appends to; repeated appends to that builder in one guard
count only once.

`tagNames` is the tag names that produce SQL text, such as `["sql"]`. An
identifier tag and a member tag (`db.sql`, `db["sql"]`, ``db[`sql`]``) both
count. A string or untagged template passed to one of those methods counts as
SQL text too.

A `const` arrow or a function declaration whose body is only that append of
its single parameter counts as the append (`const add = (text) =>
query.append(text)`). A `let`, a parameter, a second hop, or any other
statement in that function does not.

## Fix

This rule has no autofix and no editor suggestion. Replace the guards with a
switch that returns a complete statement from every arm and throws in
`default`. The diagnostic includes the guarded literal `case` labels, with
duplicates removed. Do not suggest an own-key check; that form is not judged.

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
