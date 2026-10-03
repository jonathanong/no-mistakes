# `postgres-column-naming`

Checks column names in a PostgreSQL schema snapshot against configured type
patterns, foreign-key target names, reserved suffixes, and required foreign
keys. A name should say what the column holds: timestamps end in `_at`, a
boolean reads as a predicate, and a foreign key names its target.

With only `schemaCatalogPath` set, the rule reports nothing. Type rules,
foreign-key checks, forbidden names, and `requireForeignKey` stay off until
configured. There is no built-in table or column list.

```yaml
rules:
  - rule: postgres-column-naming
    scope: repository
    options:
      schemaCatalogPath: db/schema.json
      typeRules:
        - types: ['timestamp with time zone', 'timestamp without time zone']
          namePattern: '_at$'
          hint: 'end timestamp columns in _at'
      foreignKeys:
        targetMatch: last-word
      allow:
        - object: 'column:sessions.expires'
          reason: 'Name mirrors an external protocol field'
```

## Why and when

Use this rule when a schema snapshot is the source of truth for column names.
It catches a timestamp named `due`, a boolean named `active`, a foreign key
named `item_id` that points at `articles`, and a `uuid` column named
`reviewer_user_id` that has no foreign key. Deliberate exceptions stay in
`allow` or `requireForeignKey.exempt`, each with a reason.

## What it catches/requires

`typeRules` require a column whose `data_type` is one of `types` to match
`namePattern`. `nameTypeRules` are the reverse: a matching name must use one
of the listed types. Comparison is case-insensitive. A parenthetical length
or precision, such as `character varying(2048)`, `numeric(2,-3)`, or `timestamp(6) with time
zone`, matches the base type `character varying` or `timestamp with time
zone`. An array type such as `boolean[]` does not match `boolean`. Every
matching entry reports. `forbiddenColumnNames` reports the first matching
pattern.

A single-column foreign key, other than a column named `id`, must end with a
configured `targetSuffixes` suffix when the referenced table is listed. A table is
listed by its name without SQL quoting: `Users` for `"Users"`, and `other.Users`
for a schema-qualified reference.
Otherwise, when `targetMatch` is `last-word` or `full-name`, the name must end
with the referenced column and the remaining word or words must match the
singular target name. Schema-qualified references derive the default target
name from the relation after the final dot; configured `targetNames` still
match the full relation name. Repeating a referenced column of more than one word,
such as `external_id`, skips that check only when that column itself names
the target, such as `github_account_id` on `github_accounts`. `targetNames`
can replace that singular name using `$1`
through `$9`. This check always skips composite foreign keys. It skips
self-references unless `checkSelfReferences` is set; that option does not
enable composite-key checks.

`reservedSuffixes` require the column to be the sole column of a foreign key
to one of `tables`. With `followCompositeForeignKeys`, a composite foreign key
that leads to one of those tables also counts. `requireForeignKey` requires a
matching column to appear in some foreign key. Generated columns are always
skipped by that check. Tables matching `ignoreTablePatterns` are not checked.

## Options and defaults

`schemaCatalogPath` is required, has no default, and names a catalog generated with [`no-mistakes postgres catalog`](../cli/postgres.md). `typeRules`,
`nameTypeRules`, `ignoreTablePatterns`, `forbiddenColumnNames`, and `allow`
default to `[]`. `skipGeneratedColumns` defaults to `false`.
`foreignKeys.targetMatch` defaults to `off`. `foreignKeys.checkSelfReferences`
and `foreignKeys.followCompositeForeignKeys` default to `false`.
`foreignKeys.singular` defaults to `{}`. `foreignKeys.targetSuffixes`,
`foreignKeys.reservedSuffixes`, and `foreignKeys.targetNames` default to `[]`.
`foreignKeys.requireForeignKey` defaults to unset, which turns the check off.

Empty `types`, an empty or invalid pattern, an empty hint, a duplicate
suffix, pattern, or allow object, an unknown `targetMatch`, an unknown option
key, and a `$n` past the pattern's groups fail configuration. An allow or
exempt entry needs a non-empty reason. An `include` or `exclude` list that
does not select `schemaCatalogPath` skips that catalog. A rule `message`
replaces the built-in finding text.

## Valid example

`invoices.due_at` with type `timestamp with time zone` matches `_at$`.
`projects.owner_user_id` ends in `_user_id` when that suffix is required for
`users`. `bookmarks.article_id` referencing `articles` matches `last-word`.
`import_rows.batch_id` referencing `import_batches` matches `last-word` and
fails `full-name`. `premium_accounts.id` and a self-reference
`categories.parent_id` are skipped while `checkSelfReferences` is false.

## Counterexample

`invoices.due` (`timestamp with time zone`) does not match `_at$`.
`projects.owner_id` referencing `users` does not end in `_user_id` or
`_by_id`. `bookmarks.item_id` referencing `articles` does not end in
`article_id` when `targetMatch` is `last-word`. `sessions.reviewer_user_id`
(`uuid`) with no foreign key uses a reserved `_user_id` suffix.
`comments.target_table` matches a forbidden table-name pattern.

## Fix

Rename the column so the name matches the type rule or the foreign-key target,
or add the missing foreign key. A keyset cursor, an external id, or the URL
registry's own `url` column stays as it is: add an `allow` entry or a
`requireForeignKey` exempt pattern with a reason. Drop a stale allow or exempt
entry when it no longer matches a finding or a column.

## Suppression

Schema snapshots accept strict JSON plus JSONC comments. Column findings use
the line where that column is declared, so line and next-line directives can
be placed beside the column entry. A file directive disables this rule for
the whole catalog. For example:

```jsonc
{
  "formatVersion": 2,
  "coverage": "complete",
  "tables": {
    "orders": {
      "columns": {
        "shipped_at": { // no-mistakes-disable-line postgres-column-naming: this is a calendar day
          "dataType": "timestamp with time zone"
        }
      }
    }
  }
}
```

Suppress a finding with `allow` when comments are not appropriate:

```yaml
allow:
  - object: 'column:orders.shipped_at'
    reason: 'Shipped date is a calendar day in this table'
```

One allow entry covers every finding on that object. An unused entry reports
`stale postgres-column-naming allow entry`. An unused
`requireForeignKey.exempt` pattern reports `stale postgres-column-naming
requireForeignKey exempt entry`.

## Related rules

`postgres-object-naming` checks tokens, spelling, and plural object names.
`postgres-fk-index` requires an index on a foreign key. `postgres-require-fk-on-delete`
requires an `ON DELETE` action. This rule does not rename columns or choose
an `ON DELETE` action.
