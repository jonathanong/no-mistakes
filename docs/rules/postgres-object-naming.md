<!-- cspell:ignore itms ordr -->

# `postgres-object-naming`

Checks PostgreSQL object names in a schema snapshot against configured
patterns, plural forms, denied tokens, and spelling. Naming conventions such
as `idx_<table>__<columns>` stay readable when long table names are
abbreviated: every word remains, in order, starting with its first letter.

With only `schemaCatalogPath` set, the rule reports nothing. Patterns, plural
checks, denied tokens, spelling, `tableMinWords`, and double-underscore
checks stay off until configured. Constraint-backed indexes are skipped by
default because PostgreSQL names them after their constraint.

```yaml
rules:
  - rule: postgres-object-naming
    scope: repository
    options:
      schemaCatalogPath: db/schema.json
      patterns:
        index: '^idx_{table}__[a-z0-9_]+$'
        uniqueIndex: '^(idx|uq)_{table}__[a-z0-9_]+$'
        trigger: '^trigger_[a-z0-9_]+$'
        function: '^fn_[a-z0-9_]+$'
        triggerFunction: '^fn_(reject|update|project|create|lock)_[a-z0-9_]+$'
        view: '^view_[a-z0-9_]+$'
        materializedView: '^mv_[a-z0-9_]+$'
        table: '^[a-z][a-z0-9_]*$'
        column: '^[a-z][a-z0-9_]*$'
        enum: '^[a-z][a-z0-9_]*$'
      checkConstraintBackedIndexes: false
      tableMinWords: 2
      abbreviations:
        enabled: true
        minLetters: 3
      plural:
        enabled: true
        objects: [table, enum]
        irregularPlurals: { person: people, child: children }
        uncountable: [data, metadata, feedback, media]
        nonPluralTokens: [status, analysis, sms, news, series]
        ignorePatterns: ['^link__']
      deniedTokens:
        - token: cfg
          replacement: configuration
        - token: tmp
          replacement: temporary
      spelling:
        acknowledgement: acknowledgment
      doubleUnderscore:
        allowPattern: '^link__[a-z0-9]+(_[a-z0-9]+)*__[a-z0-9_]+__[a-z0-9_]+$'
      allow:
        - object: 'table:legacy_cfg_values'
          reason: 'Name owned by an external replication tool'
```

Patterns are not anchored for you. Write `^` and `$` when the whole name
must match. `{table}` is allowed only in `index`, `uniqueIndex`, and
`trigger` patterns.

## Why and when

Use this rule when a schema snapshot is the source of truth for table, column,
index, trigger, function, view, and enum names. It keeps abbreviations from
collapsing into initialisms and keeps plural, spelling, and `__` conventions
in one place.

## What it catches/requires

Each configured `patterns` kind is applied with `Regex::is_match`. Unset kinds
are not checked. Tables and partitioned tables use `table`. Columns use
`column`. A unique index uses `uniqueIndex` when that pattern is set, and
`index` otherwise. Trigger functions must match `function` and, when set,
`triggerFunction`. Functions that return `event_trigger` use
`triggerFunction` as well. Materialized views use `materializedView` and do not fall
back to `view`. Schema-qualified snapshot keys are checked by their last
identifier. Finding targets and `allow` entries keep the full snapshot key.
An inline flag such as `(?i)` that is still active at `{table}` applies to
both sides. That includes a flag in the middle of the prefix, such as
`^idx_(?i){table}__[a-z]+$`.

`{table}` stands for the unqualified owning table. A matching abbreviation
keeps every word, in order, with the first letter and at least `minLetters`
characters taken from the word. The mismatch text suggests one abbreviation
by dropping later ASCII vowels. Tokens inside a matched `{table}` middle are
skipped only when that token also appears in the real table name. An
abbreviation-only token, such as a denied `cfg` standing in for
`configuration`, is still reported.

Plural checks, when enabled, require the last word to be plural and reject
other plural words unless they are uncountable. `tableMinWords` counts `_`
words on tables only. The suggestion adds enough `owner` and `part` prefixes
to reach that minimum. When `doubleUnderscore` is present, `__` in a table,
view, or enum must match `allowPattern`, or every `__` is reported if the
pattern is omitted.

## Options and defaults

`schemaCatalogPath` is required. `patterns.<kind>` defaults to unset.
`tableMinWords` defaults to unset and, when set, must be from 1 to 32.
`checkConstraintBackedIndexes` defaults to
`false`. `abbreviations.enabled` defaults to `false`. `abbreviations.minLetters`
defaults to `3`. `plural.enabled` defaults to `false`. `plural.objects`
defaults to `[table]`. `plural.irregularPlurals` defaults to `{}`.
`plural.uncountable` and `plural.nonPluralTokens` default to `[]`.
`plural.ignorePatterns` defaults to `[]`. `deniedTokens` defaults to `[]`.
`spelling` defaults to `{}`. `doubleUnderscore` defaults to unset.
`doubleUnderscore.allowPattern` defaults to unset when the key is present, and
then every `__` is reported. `allow` defaults to `[]`. `include`, `exclude`, and `projects` filter
`schemaCatalogPath`. A non-empty `message` replaces the finding text and keeps
the `file: object:` locator.

Unknown pattern kinds, invalid regexes, `{table}` outside index, unique index,
and trigger patterns, `tableMinWords` outside 1 to 32, `minLetters` below 1, an empty or
unknown `plural.objects` list, empty irregular keys or values, an irregular
plural equal to its singular form ignoring ASCII case, a multi-word irregular
plural, duplicate irregular keys ignoring ASCII case, empty or duplicate
denied tokens, a denied-token replacement equal to its token ignoring ASCII
case, a spelling key equal to its value ignoring ASCII case, duplicate
spelling keys ignoring ASCII case, and allow entries
with an empty reason, a duplicate object, or an invalid object ref are
configuration errors.

## Valid example

`idx_ord_line_itms__order_id` on `order_line_items` matches
`^idx_{table}__[a-z0-9_]+$` when abbreviations are enabled.
`account_people` ends in the irregular plural `people`. `orders_pkey` is
skipped while `checkConstraintBackedIndexes` stays at its default `false`.

## Counterexample

```text
schema.json: index:order_line_items.idx_oli__order_id: index name does not match pattern ^idx_{table}__[a-z0-9_]+$ ({table} = order_line_items or an abbreviation such as ordr_lin_itms)
schema.json: table:invoice_line: table name must end in a plural word; "line" is singular
schema.json: table:app_cfg_values: name uses denied token "cfg"; use "configuration"
```

## Fix

Rename the object so it matches the configured pattern, using the suggested
abbreviation when `{table}` is shortened. Pluralize the last word, replace a
denied token or misspelling, and add words until `tableMinWords` is met.
Reserve `__` for names that match `allowPattern`.

## Suppression

Snapshot findings have no source comments. Suppress one object with `allow`:

```yaml
allow:
  - object: table:accounts
    reason: core entity owned by this service
```

An allow entry that matches nothing is reported as
`stale postgres-object-naming allow entry`. `no-mistakes-disable-line`,
`no-mistakes-disable-next-line`, and `no-mistakes-disable-file` apply to
source findings; this rule reports snapshot objects, so use `allow`.

## Related rules

[`postgres-conflict-ordering`](postgres-conflict-ordering.md) and
[`postgres-lock-ordering`](postgres-lock-ordering.md) read the same schema
catalog. Column type suffixes, foreign-key names, and identifier length are
separate checks.
