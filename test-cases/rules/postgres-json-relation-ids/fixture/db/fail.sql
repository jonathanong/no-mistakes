-- The JSON key has no same-row FK write.
INSERT INTO orders (id, details)
VALUES ($1, jsonb_build_object('accountId', $2::text, 'total', 10));
UPDATE orders SET details = '{"accountId": "a1"}'::jsonb WHERE id = $1;
