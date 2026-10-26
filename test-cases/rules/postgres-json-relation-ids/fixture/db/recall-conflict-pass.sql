INSERT INTO orders (id, account_id, details)
VALUES ($1, $2, jsonb_build_object('accountId', $2))
ON CONFLICT (id) DO UPDATE SET account_id = EXCLUDED.account_id,
 details = jsonb_build_object('accountId', EXCLUDED.account_id);
