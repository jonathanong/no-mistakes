# `postgres-no-mutable-sql-registry`

A `projects: [<name>]` entry inherits [`projects.<name>.postgres`](../configuration/projects.md#postgresql-scope) when that block is set. Do not restate `importSpecifier`, `executorNames`, `executorFactoryNames`, `executorTypeNames`, or `trustedSqlTags` on the rule. `include` and `exclude` may only narrow the project scope.

Flags a member write in one module that mutates an exported SQL identifier registry declared in another module, when a configured executor interpolates that registry into a relation name. A file-local lint cannot prove that path. `no-object-mutation` still owns single-file writes.

The request prepares embedded SQL, import edges, and value-flow mutations once. This rule borrows those facts and the canonical graph. Unknown provenance and HTTP `.query` collisions are not proven violations. There is no autofix.

```yaml
rules:
  - rule: postgres-no-mutable-sql-registry
    scope: repository
    options:
      importSpecifier: "@example/db"
```

This example uses repository scope and sets `importSpecifier` on the rule. For
project scope, set it under `projects.<name>.postgres.executors` and select
that project on the rule.

## Why and when

Use this when one module exports `const tables = {}` and another assigns `tables.users = 'app.users'` before SQL reads it. `const` does not freeze members. Agents otherwise treat the registry as a closed map.

## What it catches

A known exported registry receives a member write in another file, and a configured executor interpolates that binding into a recovered SQL relation (`FROM ${tables.users}`). Findings name the producer, the writer, and the SQL sink.

Writes through a named re-export (`export { tables as registry } from "./tables"`), an `export *` barrel, or a namespace import (`import * as t` then `t.tables.users = ...`) are the same violation as a direct named import. `delete tables.users` and `Object.defineProperty(tables, ...)` are member mutations too.

Static top-level `const` bindings initialized by `require("./tables")`, including destructured `const` bindings, are checked when the actual literal route has one canonical producer. A namespace re-export (`export * as ns from "./tables"`) preserves the `ns.tables` path for both writes and SQL reads.

## Abstains

A dynamic, shadowed, conditional, or function-local `require` does not establish a registry producer. Mutable `let`/`var` require bindings also abstain, even without a visible reassignment; only retained immutable identities prove this route. When a file has multiple distinct internal `require` targets, the graph cannot associate the bound value with one canonical target, so that route abstains even if each target resolves. Ambiguous re-export routes also abstain. These cases need an explicit import before the rule can judge the cross-file write:

```ts
declare const path: string;
const registry = require(path);
registry.tables.users = "app.users";
```

For example, this route abstains because the per-file require projection sees two internal targets and cannot prove which target supplied `registry`. The same limitation applies to namespace routes with multiple local import paths:

```ts
const registry = require("./tables");
const other = require("./other");
registry.tables.users = "app.users";
```

An unrelated external require does not make another binding ambiguous. In the saved `pass-require-other-target` fixture, `tables` comes from `@example/unrelated`, while the separate bare `require("./tables")` is not bound to `tables`; no cross-file registry finding is produced:

```ts
const { tables } = require("@example/unrelated");
require("./tables");
tables.users = "app.users";
```

For example, rebinding a required namespace changes the mutation's target:

```ts
let registry = require("./tables");
registry = { tables: {} };
registry.tables.users = "app.users";
```

A computed member on the registry itself is checked, but a computed export on a module namespace may select a different registry. Without a known export path the rule abstains:

```ts
import * as registries from "./tables";
declare const key: string;
registries[key].users = "app.users";
```

Cyclic re-exports, missing producers, and arbitrary `module.exports` mutation do not prove a unique exported registry. Silence is not evidence that those registries are immutable.

## Valid example

```ts
// src/tables.ts
export const tables = { users: "app.users" } as const;
```

```ts
// src/query.ts
import { query } from "@example/db";
import { tables } from "./tables";
export function list() {
  return query(`SELECT id FROM ${tables.users}`);
}
```

A known registry root remains a registry mutation even when the member key is computed (`tables[key] = ...`). Literal computed keys are checked too. A bind-value interpolation (`WHERE name = ${tables.users}`), an HTTP `req.query(...)`, or a write to unrelated state stays silent.

## Counterexample

```ts
// src/tables.ts
export const tables: Record<string, string> = {};
```

```ts
// src/write.ts
import { tables } from "./tables";
tables.users = "app.users";
```

```ts
// src/query.ts
import { query } from "@example/db";
import { tables } from "./tables";
export function list() {
  return query(`SELECT id FROM ${tables.users}`);
}
```

`export { tables as registry } from "./tables"` then `registry.users = ...`, `export * from "./tables"` through an index, `import * as t` then `t.tables.users = ...`, `delete tables.users`, and `Object.defineProperty(tables, "users", { value: "app.users" })` are the same class of write.

## Fix

Declare the complete literal registry in one module and import it. Do not assign, delete, or redefine members after export.

```ts
export const tables = { users: "app.users" } as const;
```

## Options and defaults

`importSpecifier` defaults to empty and must be set (or set `executorNames`) to select executor calls when the rule does not inherit a project postgres block. `executorNames` defaults to `query`, `read`, and `write` when a specifier is set. `executorFactoryNames`, `executorTypeNames`, and `trustedSqlTags` default to empty. `include` and `exclude` default to empty (all indexable files).

## Effort

`effort: max` is the default and preserves cross-file registry analysis.
`effort: low` avoids the call graph, helper evaluator, and mutation/provenance
analysis. It accepts SQL written directly as a literal, or through one local
`const` initialized with that literal. Direct templates with interpolations only
in SQL value positions also qualify: their parsed relation names contain no
interpolation. These calls cannot interpolate a registry into a relation:

```ts
query("SELECT id FROM app.users");
const statement = "SELECT id FROM app.users";
query(statement);
query(`SELECT id FROM app.users WHERE id = ${id}`);
```

A registry relation such as ``query(`SELECT id FROM ${tables.users}`)`` is
unanalyzable at low effort, including when the registry uses `as const` or
`Object.freeze`. A TypeScript const assertion does not freeze runtime members;
a freeze does not establish the whole imported object's provenance or nested
immutability. Helpers, aliases, and other indirect SQL also require maximum
effort. Rewrite SQL as a literal at the executor call or one local literal
`const`, or select `max` to inspect the cross-file registry.

`unanalyzable: report` produces a finding for unsupported inputs;
`advise` places the same diagnostic in report advisories; `ignore` suppresses it.
These policies do not certify an unsupported registry immutable. The shared
repository, rule, and invocation effort/policy precedence applies. No partner
check is required because unsupported inputs are explicitly classified.

## Suppression

Use `no-mistakes-disable-next-line postgres-no-mutable-sql-registry` or
`no-mistakes-disable-line` at the executor call, or
`no-mistakes-disable-file` for a justified file exception.

## Related rules

[`postgres-no-unbounded-sql-identifier`](postgres-no-unbounded-sql-identifier.md) covers env/config identifier interpolations.
[`postgres-no-offset`](postgres-no-offset.md) covers pagination clauses.
The opt-in ESLint `no-object-mutation` rule covers single-file member writes.
