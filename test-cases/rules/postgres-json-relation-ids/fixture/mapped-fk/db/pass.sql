-- Duplicating a reference into both JSON and the mapped FK is intentional.
INSERT INTO orders (id, account_id, details)
VALUES ($1, $2, jsonb_build_object('accountId', $2::text, 'total', 10));
UPDATE orders SET account_id = $2, details = '{"accountId": "a1"}'::jsonb WHERE id = $1;
INSERT INTO orders (id, account_id, details) VALUES
($1, $2, '{"accountId": "a1"}'::jsonb),
($3, $4, jsonb_build_object('accountId', $4));
WITH source AS (SELECT $1 AS account_id)
INSERT INTO public.orders (account_id, details)
SELECT account_id, jsonb_build_object('accountId', account_id) FROM source;
WITH source AS (SELECT $1 AS account_id)
UPDATE public.orders SET account_id = source.account_id,
 details = jsonb_build_object('accountId', source.account_id) FROM source;
-- A catalog FK may contain multiple columns; the mapped reference is still explicit.
INSERT INTO composite_orders (tenant_id, account_id, details)
VALUES ($1, $2, jsonb_build_object('accountId', $2));
