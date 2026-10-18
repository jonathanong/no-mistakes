<!-- cspell:ignore functiondef viewdef constraintdef triggerdef -->

# PostgreSQL fact sources

`no-mistakes` exposes two reusable, rule-free fact sources for PostgreSQL
work. Check rules consume these facts instead of re-parsing SQL or
TypeScript.

The existing prepared repository extractors below are Rust library APIs.
For standalone SQL text, the public async Node
[`parsePostgresSql`](postgres-source-api.md) exposes versioned typed source
facts without a repository scan or database. It shares the same parser and
publishes partial diagnostics and source coordinates.
`postgres-conflict-ordering`, `postgres-lock-ordering`,
`postgres-column-requires-trigger`, `postgres-required-comments`,
`postgres-duplicate-function-body`, `postgres-table-shape`,
`postgres-status-with-lifecycle-timestamps`,
`postgres-no-offset`,
`postgres-require-query-annotation`,
`postgres-no-generated-column-writes`,
`postgres-fk-index`, `postgres-redundant-index`,
`postgres-constraint-validate`, `postgres-no-add-column`,
`postgres-column-naming`, `postgres-finite-text-columns`,
`postgres-array-columns`, `postgres-key-column-types`, and
`postgres-identifier-length`, `postgres-required-predicates`, and
`postgres-sql-shape-policy`
consume the facts through `no-mistakes check`. Forthcoming DML rules
(`postgres-idempotent-insert`) will consume the same INSERT/SELECT facts
once registered.

See [mutable SQL builders and conservative recovery](postgres-builder-mutations.md)
for alias invalidation and escape boundaries.

## Schema facts

See [Schema facts](postgres-facts/schema-facts.md).

## Embedded-SQL facts

See [Embedded-SQL facts](postgres-facts/embedded-sql-facts.md).

### Finite SQL alternatives

See [Finite SQL alternatives](postgres-facts/finite-sql-alternatives.md).

See also [builder versions and conservative recovery](postgres-facts/builder-versions.md).

### Imported and literal-identifier helpers

See [Imported and literal-identifier helpers](postgres-facts/imported-and-literal-identifier-helpers.md).

#### Helper control flow and nested composition

See [Imported and literal-identifier helpers](postgres-facts/imported-and-literal-identifier-helpers.md).

#### Executor templates over a builder's `.text`

See [Imported and literal-identifier helpers](postgres-facts/imported-and-literal-identifier-helpers.md).

#### Fluent statement chains

See [Imported and literal-identifier helpers](postgres-facts/imported-and-literal-identifier-helpers.md).

#### What stays `Dynamic`

See [Imported and literal-identifier helpers](postgres-facts/imported-and-literal-identifier-helpers.md).

#### Helper-line attribution and suppression

See [Imported and literal-identifier helpers](postgres-facts/imported-and-literal-identifier-helpers.md).

### Executor bindings

See [Executor bindings](postgres-facts/executor-bindings.md).

## Statement facts

See [Statement facts](postgres-facts/statement-facts.md).

## Locking-select facts

See [Locking-select facts](postgres-facts/locking-select-facts.md).

## Offset facts

See [Offset facts](postgres-facts/offset-facts.md).

## Row-bound facts

See [Row-bound facts](postgres-facts/row-bound-facts.md).

## Query annotation facts

See [Query annotation facts](postgres-facts/query-annotation-facts.md).

## Schema catalog model

See [Schema catalog model](postgres-facts/schema-catalog-model.md).

## Out of scope

See [Out of scope](postgres-facts/out-of-scope.md).

## Generated schema catalogs

See [Generated schema catalogs](postgres-facts/generated-schema-catalogs.md).
