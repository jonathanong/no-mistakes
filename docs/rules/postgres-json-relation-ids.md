<!-- cspell:ignore bytea -->

# `postgres-json-relation-ids`

A `projects: [<name>]` entry inherits [`projects.<name>.postgres`](../configuration/projects.md#postgresql-scope) when that block is set. Do not restate `importSpecifier`, `executorNames`, `executorFactoryNames`, `executorTypeNames`, or `trustedSqlTags` on the rule. `include`, `exclude`, and `sqlInclude` may only narrow the project scope.

This opt-in PostgreSQL check finds row identifiers stored in top-level `jsonb`
keys without a same-row foreign key, and unique fingerprint columns that may be
standing in for relational keys. It is advisory: findings are returned in
`advisories` and never fail a run by default.

## Why and when

Use this rule when row references can be hidden in JSON or folded into a hash.
PostgreSQL cannot enforce those relationships or apply foreign-key delete
behavior, and queries must extract the identifiers from documents. A normal
foreign key gives the database and cleanup queries an explicit relationship.

## Options and defaults

```yaml
rules:
  - rule: postgres-json-relation-ids
    scope: repository
    options:
      schemaCatalogPath: db/schema.json
      idKeyPattern: '(^id$|Id$|_id$)'
      keyColumnMap:
        accountId: account_id
        orderId: order_id
      fingerprintNamePattern: '_sha256$'
      allow:
        - object: 'table:invoice_revisions'
          reason: 'append-only history document; nothing joins or cleans up by these keys'
```

| Option | Type | Default | Meaning |
| --- | --- | --- | --- |
| `schemaCatalogPath` | string | none | Required when `keyColumnMap` or `fingerprintNamePattern` is set. |
| `idKeyPattern` | regex string | unset | Selects JSON keys for check 1. Unset turns off this check; an empty string is invalid. |
| `keyColumnMap` | map of JSON key to column | `{}` | Candidate foreign-key column for a JSON key. A mapped column only excuses the key when the same statement writes that column and the schema catalog identifies it as a foreign key on that table. |
| `fingerprintNamePattern` | regex string | unset | Selects unique `bytea` or `text` columns for check 2. Unset turns off this check; an empty regex matches every column name. |
| `allow` | list of `{object, reason}` | `[]` | Allows `table:<name>` or `column:<table>.<column>` with a nonempty reason. A table exception is limited to that table. |
| `include`, `exclude` | string arrays | rule defaults | Narrow TypeScript/JavaScript executor files. |
| `sqlInclude` | string array | `**/*.sql` | Selects migration/schema SQL files to inspect. |
| `importSpecifier`, `executorNames`, `executorFactoryNames`, `executorTypeNames`, `trustedSqlTags` | standard PostgreSQL executor options | project defaults | Select executed embedded SQL. Configure the import and executor profile as for other PostgreSQL rules. |

With no options the rule reports nothing. It assumes no key naming convention.
Generate `schemaCatalogPath` with [`no-mistakes postgres catalog`](../cli/postgres.md).
Invalid regexes, an empty `idKeyPattern`, empty map keys or values, duplicate
map keys, malformed allow objects, and empty allow reasons are configuration
errors. An empty `fingerprintNamePattern` is valid and matches every name. A
map entry whose column is not a foreign key is valid configuration, but does
not excuse a JSON key in a finding.

## What it catches

For `INSERT` and `UPDATE` statements in selected migration SQL and statically
recovered executed embedded SQL, the rule examines top-level keys written by
`jsonb_build_object('key', value, ...)` or a literal object cast to `jsonb`. A
key matching `idKeyPattern` is advisory unless the same statement also writes
its mapped foreign-key column for the same table, or an `allow` entry covers
the table or JSON column. Duplicating an ID in JSON and in its mapped foreign
key column is allowed.

Nested keys and non-literal dynamic keys are out of scope and skipped. Dynamic
embedded SQL that cannot be analyzed produces no finding. This rule does not
infer a JSON document format.

Separately, a `bytea` or `text` column whose name matches
`fingerprintNamePattern` is reported for each unique constraint or unique index
that includes it. A matching column outside a unique key is clean.

## Valid example

The examples use this excerpt from the configured catalog:

```json
// db/schema.json
{
  "formatVersion": 2,
  "coverage": "complete",
  "tables": {
    "accounts": {"columns": {"id": {"dataType": "uuid"}}},
    "orders": {
      "columns": {"id": {"dataType": "uuid"}, "account_id": {"dataType": "uuid"}, "details": {"dataType": "jsonb"}},
      "foreignKeys": {"orders_account_fkey": {"columns": ["account_id"], "referencedTable": "accounts", "referencedColumns": ["id"]}}
    },
    "invoices": {
      "columns": {"id": {"dataType": "uuid"}, "account_id": {"dataType": "uuid"}, "number": {"dataType": "text"}, "fingerprint_sha256": {"dataType": "bytea"}}
    }
  }
}
```

The JSON value duplicates an actual foreign key in the same `INSERT`:

```sql
-- db/orders.sql
CREATE TABLE orders (
  id uuid PRIMARY KEY,
  account_id uuid NOT NULL REFERENCES accounts (id) ON DELETE CASCADE,
  details jsonb NOT NULL
);

INSERT INTO orders (id, account_id, details)
VALUES ($1, $2, jsonb_build_object('accountId', $2::text, 'total', 10));
```

Here `keyColumnMap.accountId` is `account_id`, and the catalog confirms that
`orders.account_id` references another table. The same-statement write proves
the JSON identifier has a relational counterpart. For positional `INSERT`s, the
catalog must supply column ordinals to prove that write. Use an explicit column
list when ordinals are unavailable.

```sql
-- db/invoices.sql
CREATE TABLE invoices (
  id uuid PRIMARY KEY,
  account_id uuid NOT NULL REFERENCES accounts (id),
  number text NOT NULL,
  fingerprint_sha256 bytea NOT NULL,
  CONSTRAINT invoices_account_number_key UNIQUE (account_id, number)
);
```

The fingerprint column is not part of a unique key, so check 2 is clean.

```sql
-- db/history.sql
-- This table's history document is explicitly excepted by configuration.
INSERT INTO invoice_revisions (id, invoice_id, changes)
VALUES ($1, $2, '{"accountId": "a1"}'::jsonb);
```

## Counterexample

Here the catalog also contains the fingerprint unique key:

```json
// db/schema.json
{
  "formatVersion": 2,
  "coverage": "complete",
  "tables": {
    "accounts": {"columns": {"id": {"dataType": "uuid"}}},
    "orders": {
      "columns": {"id": {"dataType": "uuid"}, "account_id": {"dataType": "uuid"}, "details": {"dataType": "jsonb"}},
      "foreignKeys": {"orders_account_fkey": {"columns": ["account_id"], "referencedTable": "accounts", "referencedColumns": ["id"]}}
    },
    "invoices": {
      "columns": {"id": {"dataType": "uuid"}, "account_id": {"dataType": "uuid"}, "number": {"dataType": "text"}, "fingerprint_sha256": {"dataType": "bytea"}},
      "uniqueConstraints": {"invoices_fingerprint_key": {"columns": ["fingerprint_sha256"]}}
    }
  }
}
```

`accountId` matches the configured pattern, but there is no same-row mapped
foreign key write:

```sql
-- db/orders.sql
INSERT INTO orders (id, details)
VALUES ($1, jsonb_build_object('accountId', $2::text, 'total', 10));
```

The advisory identifies `orders.details` and recommends writing the mapped
foreign-key column or adding a reasoned exception.

```sql
-- db/invoices.sql
CREATE TABLE invoices (
  id uuid PRIMARY KEY,
  account_id uuid NOT NULL REFERENCES accounts (id),
  fingerprint_sha256 bytea NOT NULL,
  CONSTRAINT invoices_fingerprint_key UNIQUE (fingerprint_sha256)
);
```

With `fingerprintNamePattern: '_sha256$'`, the fingerprint participates in a
unique key and is reported. Prefer a unique key over the relational foreign-key
columns.

## Findings

Example messages:

```text
db/write.sql:12: (postgres-json-relation-ids) key "accountId" written into orders.details has no same-row foreign key column; write account_id with a foreign key, or allow the column with a reason
db/schema.json:8: (postgres-json-relation-ids) column invoices.fingerprint_sha256 matches fingerprintNamePattern and is part of UNIQUE key invoices_fingerprint_key; key on foreign key columns instead
```

Each finding references `table:<table>` or `column:<table>.<column>`. Results
are sorted deterministically. The same fingerprint column in two unique keys
produces one advisory for each key.

## Fix and suppression

Write the real foreign-key column in the same statement as the JSON document,
and make sure that column is a foreign key in the schema catalog. For a
fingerprint advisory, use relational key columns in the unique constraint.
If the document intentionally retains identifiers only for a reason that does
not need a relation, add a narrowly scoped `allow` entry with a useful reason.

SQL comments can suppress an individual finding with:

```sql
INSERT INTO invoice_revisions (id, invoice_id, changes)
-- no-mistakes-disable-next-line postgres-json-relation-ids -- retained solely for audit history
VALUES ($1, $2, '{"accountId": "a1"}'::jsonb);
```

Use `-- no-mistakes-disable-line postgres-json-relation-ids` for a finding on
the same line, or `-- no-mistakes-disable-file postgres-json-relation-ids` at
the top of a SQL file to opt out of the rule for that file. All three
suppression forms also work for findings in executed embedded SQL.

## CLI and Node API

Configure the rule and run `no-mistakes check --root . --format json`. Its
findings are advisories and do not change the default check exit status. The
async Node `check()` API and `analyzeProject()` check reports use the same
configuration and advisory behavior. See [Node API mapping](../node-api.md).

## Related rules

[`postgres-fk-index`](postgres-fk-index.md) checks indexes for declared foreign
keys; [`postgres-key-column-types`](postgres-key-column-types.md) checks their
types. [`postgres-column-naming`](postgres-column-naming.md) enforces configured
column naming policies. This rule covers references hidden in JSON or unique
fingerprints that those declared-column checks cannot see.
