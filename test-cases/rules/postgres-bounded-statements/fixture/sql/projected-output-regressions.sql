-- Composite element attributes belong to UNNEST, even without explicit output aliases.
DELETE FROM accounts WHERE id IN (SELECT id FROM unnest($1::account_row[]) u LIMIT 1);
DELETE FROM accounts WHERE id IN (SELECT id FROM pg_catalog.unnest($1::account_row[]) u LIMIT 1);
DELETE FROM accounts WHERE id IN (SELECT id FROM unnest($1::account_row[]) u(other, id) LIMIT 1);
-- Ordinary scalar functions add ordinality after their scalar value column.
DELETE FROM accounts WHERE id IN (SELECT $1::uuid FROM generate_series(1,3) WITH ORDINALITY WHERE ordinality > 0 LIMIT 1);
DELETE FROM accounts WHERE id IN (SELECT $1::uuid FROM generate_series(1,3) WITH ORDINALITY g(value) WHERE ordinality > 0 LIMIT 1);
DELETE FROM accounts WHERE id IN (SELECT $1::uuid FROM generate_series(1,3) WITH ORDINALITY g(value,n) WHERE n > 0 LIMIT 1);
-- A complete join wrapper alias replaces child output labels.
DELETE FROM accounts WHERE id IN (SELECT id FROM ((SELECT $1::uuid AS code) c CROSS JOIN (SELECT $2::uuid AS symbol) d) j(id,symbol) LIMIT 1);
-- Known scalar arrays still expose precise names and protect outer-column correlation.
DELETE FROM accounts WHERE id IN (SELECT id FROM unnest($1::uuid[][]) u LIMIT 1);
DELETE FROM accounts WHERE id IN (SELECT id FROM unnest($1::uuid ARRAY) u LIMIT 1);
DELETE FROM accounts WHERE id IN (SELECT id FROM unnest(ARRAY[1,2]) u LIMIT 1);
DELETE FROM accounts WHERE id IN (SELECT id FROM unnest(ARRAY[ARRAY[1],ARRAY[2]]) u LIMIT 1);
-- Unknown constructors and intentionally unsupported array annotations retain opaque ownership.
DELETE FROM accounts WHERE id IN (SELECT id FROM unnest(ARRAY[$1]) u LIMIT 1);
DELETE FROM accounts WHERE id IN (SELECT id FROM unnest(ARRAY[ROW($1,$2)]) u LIMIT 1);
DELETE FROM accounts WHERE id IN (SELECT id FROM unnest($1) u LIMIT 1);
DELETE FROM accounts WHERE id IN (SELECT id FROM unnest($1::ARRAY<uuid>) u LIMIT 1);
DELETE FROM accounts WHERE id IN (SELECT id FROM unnest(ARRAY[(SELECT id FROM accounts LIMIT 1)]) u LIMIT 1);
-- Qualified scalar UNNEST and nondefault ordinality aliases remain precise.
DELETE FROM accounts WHERE id IN (SELECT id FROM pg_catalog.unnest($1::uuid[]) u LIMIT 1);
DELETE FROM accounts a(id,email,ordinality) WHERE id IN (SELECT $1::uuid FROM generate_series(1,3) WITH ORDINALITY g(value,n) WHERE ordinality IS NOT NULL LIMIT 1);
