-- Other-table FKs and columns without FKs do not establish this row's relation.
INSERT INTO loose_orders (account_id, details) VALUES ($1, jsonb_build_object('accountId', $1));
INSERT INTO orders (alternate_id, details) VALUES ($1, '{"accountId": "a1"}'::jsonb);
UPDATE loose_orders SET account_id = $1, details = jsonb_build_object('accountId', $1);
