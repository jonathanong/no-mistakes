UPDATE orders SET account_id = $2,
 details = CASE WHEN $1 THEN jsonb_build_object('accountId', $2) ELSE '{}'::jsonb END;
INSERT INTO orders (account_id, details)
VALUES ($1, COALESCE(NULL, jsonb_build_object('accountId', $1)));
