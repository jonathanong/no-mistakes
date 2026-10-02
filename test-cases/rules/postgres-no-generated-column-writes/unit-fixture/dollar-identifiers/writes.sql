SELECT foo$tag$bar, metric$tag$ FROM other_table;
UPDATE orders SET updated_at = now();
-- no-mistakes-disable-next-line postgres-no-generated-column-writes
UPDATE orders SET updated_at = now();
SELECT café$tag$bar, foo$$bar, $1$not_a_quote FROM other_table;
UPDATE orders SET updated_at = now();
DO $café$ BEGIN UPDATE orders SET updated_at = now(); END $café$;
DO $東京$ BEGIN UPDATE orders SET updated_at = now(); END $東京$;
