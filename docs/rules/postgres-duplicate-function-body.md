# `postgres-duplicate-function-body`

Functions whose bodies are the same once names and literals are normalized are
reported as one cluster. A single function, a body below `minTokens`, or the
same text in two languages produces nothing. `schemaCatalogPath` is required;
enabling the rule without it is a configuration error. With only that path
set, the defaults report every cluster of at least two functions.

```yaml
rules:
  - rule: postgres-duplicate-function-body
    scope: repository
    options:
      schemaCatalogPath: db/schema.json
      minClusterSize: 2
      minTokens: 1
      normalizeIdentifiers: true
      normalizeRaise: true
      keepIdentifiers: ['now', 'coalesce']
      allow:
        - object: function:fn_orders_audit
          reason: Kept separate on purpose; audited by a different team
```

## Why and when

Schemas grow near-identical functions that differ only by a table name, a
column name, or the error text. Each copy has to be fixed separately, and the
copies drift. One function that uses `TG_TABLE_NAME` or `TG_ARGV` replaces
them.

## What it catches/requires

The rule reads function bodies from the schema catalog. A function with no
body, or a body that fails to tokenize, is skipped. `AS '...'` is a body, and
a doubled quote inside it is one quote. Whitespace and comments are dropped,
and a trailing `;` is dropped so `END;` and `END` match. A default or comment
that contains `returns event_trigger` does not make the function an event
trigger.

When `normalizeRaise` is true, each `RAISE` statement becomes
`RAISE <level> ? ;`. The level is `EXCEPTION`, `WARNING`, `NOTICE`, `INFO`,
`LOG`, or `DEBUG`, and defaults to `EXCEPTION`. The message, format arguments,
and `USING` options are dropped. A `;` inside a string does not end the
statement.

String literals, including dollar quotes, become `'?'`. Numbers become `0`.
When `normalizeIdentifiers` is true, non-keyword words and quoted identifiers
become `ID`, except a word immediately followed by `(` which is kept. SQL
keywords, `NEW`, `OLD`, words starting with `TG_`, `keepIdentifiers`, and
operators stay in uppercase where they are words.

The group key is the language, that token sequence, the search path, security,
parallel mode, null-input behavior, volatility, leakproof, the return contract,
whether the body is a string or a parsed `BEGIN ATOMIC` / `RETURN` expression,
and whether the routine is a trigger, an event trigger, or an ordinary
function. Every member of a
group of size at least `minClusterSize`, with at least `minTokens` tokens, is
reported as `function:<snapshot key>`. The text lists the other keys in
snapshot-key order, at most five, then `and <m> more`, and says the copies
drift so a fix has to be repeated in each copy.

## Options and defaults

`schemaCatalogPath` is required. `minClusterSize` defaults to `2` and must be
at least 2. `minTokens` defaults to `1` and must be at least 1.
`normalizeIdentifiers` and `normalizeRaise` default to `true`.
`keepIdentifiers` and `allow` default to `[]`. An empty allow reason, an
invalid object ref, or a duplicate allow object fails configuration.
`include`, `exclude`, and `projects` apply to the schema catalog path. Only
`sql` and `plpgsql` bodies are compared; a missing language is compared only
with other bodies that also omit it. Unquoted call names fold;
quoted call names and schema qualifiers stay distinct. Other identifiers
become stable placeholders, so a consistent rename still matches and a swapped
use does not. Different `SET search_path` clauses and `SECURITY DEFINER` versus invoker do
not match. The body is located by its delimiter, so an earlier copy of the
body text does not hide those clauses. `STRICT` and `RETURNS NULL ON NULL INPUT`
are a separate group from `CALLED ON NULL INPUT` and the default. `IMMUTABLE`,
`STABLE`, and `VOLATILE` are separate groups. Those words count only after the
argument list, so a parameter named `immutable` stays in the default group. A
`search_path` clause also stops at those words. Different return
contracts stay separate, including `integer` versus `integer[]`. `PARALLEL SAFE`,
`PARALLEL RESTRICTED`, and the default `PARALLEL UNSAFE` are separate groups.
A dollar-quote tag is empty or an identifier, and `$` inside an identifier is
not a quote. `SET` and `search_path` may be separated by whitespace or a
comment. Quoted return types keep their case. `LEAKPROOF` is a separate group
from the default. `SET search_path TO security, definer` is not `SECURITY
DEFINER`. `IF NOT EXISTS` does not hide a `BEGIN ATOMIC` body. A dollar-quoted
default is not a `search_path` clause. `OUT` and `INOUT` parameters stay in
the return contract. A kept identifier stays quoted when the function quotes it.
A `SET` value is not a function mode, and a `returns` call in a parameter
default is not the return contract. Escape strings decode hex, octal, and
Unicode sequences. `AS U&'...'` is a body. A qualified `raise(...)` call stays
a call. An escape-string default does not hide the real body. Names before `=>` or `:=` stay distinct. `AS E'...'` is a body, and adjacent string literals are one body.
`LANGUAGE SQL RETURN expression` is a body. `LANGUAGE` after `BEGIN ATOMIC`
is the function language, not a word in the body. A comment or literal that
says `SECURITY DEFINER` does not make the function a definer. Quoted
`search_path` entries keep their case. Trigger functions, event
triggers, and ordinary functions are separate groups. Bare `RAISE;` stays
distinct from `RAISE EXCEPTION`. A top-level `message` replaces the generated
finding text.

## Valid example

One `fn_reject_mutation()` that raises `'% is immutable', TG_TABLE_NAME`.
`PERFORM fn_touch_orders()` and `PERFORM fn_touch_invoices()` stay distinct
because the called name is kept. The same body in `sql` and `plpgsql` does not
match. `minTokens: 12` ignores the six-token always-raise copies.

## Counterexample

`fn_reject_orders_update`, `fn_reject_invoices_update`, and
`fn_reject_refunds_update` all normalize to `BEGIN RAISE EXCEPTION ? ; END`,
including a copy that uses `USING ERRCODE`, a format argument, or `END;`.
`NEW.title IS DISTINCT FROM OLD.title` matches `NEW.body IS DISTINCT FROM
OLD.body` while `normalizeIdentifiers` is true.

## Fix

Replace trigger copies with one function parameterized by `TG_TABLE_NAME`
or `TG_ARGV`. Replace other copies with one function that takes the varying
values as arguments. Raise the `minTokens` floor when the shared shape is too small to
be worth merging. Set `normalizeIdentifiers` or `normalizeRaise` to false when
those differences are intentional.

## Suppression

Snapshot JSON cannot carry `no-mistakes-disable` comments. Suppress one
function with `allow: [{object, reason}]`, where `object` is
`function:<snapshot key>` and `reason` is non-empty. An entry that matches
nothing is reported as a stale allow entry.

## Related rules

`postgres-column-requires-trigger` reads trigger definitions from the same
schema catalog. This rule reads function bodies and does not compare them to
the triggers that call them.
