# `no-mistakes/postgres-cursor-call-contract`

## Why

Cursor executors hold a database client for a stream's lifetime. Direct,
annotated static SQL lets review attribute that cost to one callsite.

## Disallowed

```ts
const cursor = runCursor;
cursor(sql);
runCursor("SELECT * FROM users");
```

## Allowed

```ts
import { runCursor } from "@app/db/cursors";
runCursor("/* users */ SELECT * FROM users");
```

## Options

- `modules` and `executors` identify cursor imports and required executor names;
  either empty list disables the rule.
- `include`, `exclude`, and `includeFiles` scope files. `include` defaults to
  `**/*.{ts,mts,tsx,js,mjs}`; `includeFiles` wins over `exclude`.
- `annotation` replaces the leading `/* name */` requirement.
- `sqlTagModules` lists supported SQL tag modules and defaults to
  `["sql-template-strings"]`.

## Fix

Call a configured imported executor directly with an annotated literal, static
template, supported SQL tag, or one immutable local binding.

## Suppression

```ts
// eslint-disable-next-line no-mistakes/postgres-cursor-call-contract -- compatibility adapter passes a validated query object
runCursor(query);
```

## Related rules

- [`postgres-no-manual-transaction`](postgres-no-manual-transaction.md) keeps
  transaction lifecycle in the owning helper.
