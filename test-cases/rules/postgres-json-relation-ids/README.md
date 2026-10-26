# postgres-json-relation-ids fixtures

Saved catalog, migrations, and executed embedded SQL exercise top-level JSON relation keys,
explicit same-row FK writes, table/column allowances, physical source locations, suppression,
and fingerprint columns in unique constraints and indexes. `count_sha256` is deliberately
an integer and `unused_sha256` deliberately has no unique key: neither should report.

`db/pass.sql` includes schema-qualified targets, CTEs, multiple rows, and a composite FK.
`db/wrong-fk.sql` keeps the same column name on another table without an FK so catalog
lookups cannot accidentally borrow a relation from `orders`.

`no-mistakes.low.yml` enables only JSON key detection, and `no-mistakes.max.yml` combines
both checks and an explicit history-table allowance. The duplicate YAML/JSONC configurations
must fail before a map parser discards repeated `accountId` keys.
