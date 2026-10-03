SELECT 1 FROM accounts WHERE id = $1 LIMIT 1;
SELECT id FROM orders WHERE account_id = $1 ORDER BY id LIMIT 20;
SELECT code FROM currencies ORDER BY code LIMIT $1;
SELECT code FROM public.currencies WHERE code > $1 ORDER BY code LIMIT $2;
SELECT id FROM orders WHERE deleted_at = false AND id > $1 ORDER BY id LIMIT $2;
SELECT id FROM orders WHERE id > $1 ORDER BY id LIMIT 7;
