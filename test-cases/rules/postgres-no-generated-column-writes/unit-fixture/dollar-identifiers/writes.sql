SELECT foo$tag$bar FROM other_table;
UPDATE orders SET updated_at = now();
-- no-mistakes-disable-next-line postgres-no-generated-column-writes
UPDATE orders SET updated_at = now();
SELECT café$tag$bar, foo$$bar, $1$not_a_quote FROM other_table;
UPDATE orders SET updated_at = now();
