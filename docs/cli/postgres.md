<!-- cspell:ignore libpq dbname sslmode sslrootcert sslcert sslkey PGSERVICE PGSERVICEFILE PGHOSTADDR -->
# `postgres`

## `postgres catalog`

Generate no-mistakes-owned ordering facts directly from a PostgreSQL database.
No application snapshot producer or snapshot format is required.

```sh
no-mistakes postgres catalog --connection-env DATABASE_URL --schema public --output ordering-catalog.json
```

Set the named environment variable to a PostgreSQL connection URL. The connection
is passed through the child environment, never an argument or diagnostic.
Install `psql`; libpq authentication and TLS configuration apply. The generator
uses a single repeatable-read, read-only transaction and does not execute DDL.
The schema is an exact name, including case, not an SQL expression. A missing
schema or failed query is an error, rather than an empty proof.

The deterministic JSON uses the tool's `formatVersion: 2` contract and explicit
`coverage: "ordering"`. It includes observed table columns, primary/unique
constraints and indexes, including validity/readiness, key expressions, direction,
NULL ordering and predicates. INCLUDE columns do not become ordering keys.
Invalid, not-ready or non-live indexes remain visible and cannot prove order.
Deferrable indexes cannot resolve conflict arbiters. Non-default operator classes or
collations the canonical comparator cannot model have `orderingSupported: false`
and fail closed; their observed metadata remains in the catalog.

Use the output as `schemaCatalogPath` for `postgres-conflict-ordering` and
`postgres-lock-ordering`. Other catalog-backed rules require complete schema facts
and reject this partial catalog. Omitted checks, functions, triggers and other
schema facts are not asserted absent. Existing complete catalogs without a
coverage field remain complete. Regenerate after applying schema changes to a
fresh test database; the catalog describes observed state at generation time.

The async Node equivalent is `generatePostgresCatalog({ connectionEnv, schema })`.
It returns the catalog object; the caller owns writing it to disk.

Supported URL query parameters are `host`, `port`, `user`, `dbname`, `sslmode`,
`sslrootcert`, `sslcert`, `sslkey` and `connect_timeout`. Other libpq configuration
can use its standard `PG*` environment variables; unsupported URL parameters are
errors. URL components override corresponding inherited environment settings.

Inherited `PGSERVICE`, `PGSERVICEFILE` and `PGHOSTADDR` are cleared so they cannot
override the selected URL. Other omitted libpq settings retain standard defaults.

Requires PostgreSQL 12 or newer and `psql`. Ordering proof supports default
operator classes and column indexes whose collation matches the column.
Expression indexes with collations remain unprovable; their metadata is retained.
Any unsupported or deferrable matching conflict index prevents proof, even when
another matching index is supported.
