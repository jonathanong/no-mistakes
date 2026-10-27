UPDATE orders SET (details, alternate_id) = (jsonb_build_object('accountId', $1), $1);
INSERT INTO orders (details)
SELECT jsonb_build_object('accountId', $1)
UNION ALL SELECT '{"accountId":"a2"}'::jsonb;
INSERT INTO orders (details) VALUES (JSONB '{"accountId":"a1"}');
INSERT INTO orders (details) VALUES ($doc${"accountId":"a1"}$doc$::jsonb);
UPDATE orders SET details = '{"accountId":"a1"}'::jsonb || jsonb_build_object('orderId', $1);
INSERT INTO orders (details) VALUES (jsonb_build_object(CAST('accountId' AS text), $1));
INSERT INTO orders (details) VALUES (pg_catalog.jsonb_build_object('accountId', $1));
