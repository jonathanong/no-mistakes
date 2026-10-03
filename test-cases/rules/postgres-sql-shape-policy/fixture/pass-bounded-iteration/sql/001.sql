SELECT id FROM orders WHERE next_reconcile_at <= now() ORDER BY next_reconcile_at, id LIMIT $1;
SELECT id FROM orders WHERE account_id = $1 AND id > $2 ORDER BY id LIMIT $3;
SELECT id FROM order_reconcile_work_items WHERE lease_expires_at IS NULL ORDER BY id LIMIT $1;
SELECT 1 FROM accounts WHERE id = $1 LIMIT 1;
SELECT id FROM orders WHERE ($1::uuid IS NULL OR id > $1) AND deleted_at IS NULL ORDER BY id LIMIT $2;
SELECT id FROM orders ORDER BY id;
SELECT id, count(*) FROM orders GROUP BY id ORDER BY id LIMIT $1;
