UPDATE orders SET (details, account_id) = (jsonb_build_object('accountId', $1), $1);
-- Positional inserts follow catalog ordinals, including the mapped FK write.
INSERT INTO orders VALUES ($1, $2, $3, JSONB '{"accountId":"a1"}', NULL, NULL);
INSERT INTO orders (details) VALUES ('not-json'::jsonb), ('[{"accountId":"a1"}]'::jsonb);
