-- Standalone marker-shaped identifiers remain ordinary column uses.
SELECT id FROM orders WHERE sql_placeholder_1 = $1 AND "sql_placeholder_1" = $2 ORDER BY sql_placeholder_1;
UPDATE orders SET status = 'x' WHERE sql_placeholder_1 = $1;
DELETE FROM orders WHERE "sql_placeholder_1" = $1;
