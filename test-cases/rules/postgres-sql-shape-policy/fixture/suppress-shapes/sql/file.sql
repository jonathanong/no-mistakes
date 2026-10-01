-- no-mistakes-disable-file postgres-sql-shape-policy
SELECT COUNT(*) > 0 AS has_orders FROM orders WHERE account_id = $1;
