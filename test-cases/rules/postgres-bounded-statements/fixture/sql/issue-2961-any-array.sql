-- One scalar element is equality with that value.
WITH c AS (SELECT id FROM orders ORDER BY id LIMIT 1)
UPDATE orders SET account_id = $2 FROM c WHERE orders.id = ANY(ARRAY[c.id]);

WITH c AS (SELECT id FROM orders ORDER BY id LIMIT 1)
UPDATE orders SET account_id = $2 FROM c WHERE orders.id = ANY(ARRAY[(c.id)]);

WITH c AS (SELECT id FROM orders ORDER BY id LIMIT 1)
UPDATE orders SET account_id = $2 FROM c WHERE orders.id = ANY(ARRAY[c.id::uuid]);
