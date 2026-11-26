# `no-mistakes/postgres-cursor-call-contract`

## Why

Cursor executors hold a database client for a stream's lifetime. Direct,
annotated static SQL lets review attribute that cost to one callsite.

## Disallowed

```ts
import { runCursor } from "@app/db/cursors";

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

```js
{
  "no-mistakes/postgres-cursor-call-contract": [
    "error",
    { modules: ["@app/db/cursors"], executors: ["runCursor"] },
  ],
}
```

- `modules` and `executors` identify cursor imports and required executor names;
  either empty list disables the rule. TypeScript `import ns = require("...")`
  bindings from a configured module are checked the same as `import * as ns from
  "..."`. `const cur = require("...")` and `const { runCursor } = require("...")`
  are checked the same way. The binding declaration is not a use.
  `export import ns = require("...")` is rejected; `import type ns =
  require("...")` is ignored. A dynamic specifier is `unanalyzable` rather than
  a trusted cursor. A parenthesized callee and a type-argument callee
  (`(openCursor<Row>)("/* ok */ SELECT 1")`) unwrap to the same call.
- `include`, `exclude`, and `includeFiles` scope files. `include` defaults to
  `**/*.{ts,mts,tsx,js,mjs}`; `includeFiles` wins over `exclude`.
- `annotation` replaces the leading `/* name */` requirement.
- `sqlTagModules` lists supported SQL tag modules and defaults to
  `["sql-template-strings"]`. A default import from one of those modules is a
  trusted tag.
- `trustedSqlTags` defaults to empty. Each entry `{ module, name }` trusts a
  named import of `name` from `module` or a subpath of `module` (`@example/db/sql`
  matches `@example/db`; `@example/dbx` does not), including a rename such as
  `import { sql as dbSql }`. A default import is not enough. A shadowed or
  rebound local fails closed. The same name from another module stays untrusted.
  An empty `module` or `name` matches nothing. A namespace member
  (`import * as s` then `s.sql`) and `const { sql } = require(module)` match
  the same entry. A parenthesized type-argument tag is that tag. An unresolved
  tag is not trusted.
- `unanalyzable` is `advise` (the default), `report`, or `ignore`. `advise` and
  `report` use the same message. `ignore` omits only that finding.

## Fix

Call a configured imported executor directly with an annotated literal, static
template, supported SQL tag, or one immutable local binding.

## Suppression

```ts
// no-mistakes-disable-next-line postgres-cursor-call-contract -- compatibility adapter passes a validated query object
runCursor(query);
```

`no-mistakes-disable-line` on the finding and a leading `no-mistakes-disable-file`
work the same way. Standard ESLint and Oxlint disable comments also apply.

## Related rules

- [`postgres-no-manual-transaction`](postgres-no-manual-transaction.md) keeps
  transaction lifecycle in the owning helper.
