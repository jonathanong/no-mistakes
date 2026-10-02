-- A joined target is not one physical base relation.
UPDATE (orders JOIN accounts ON orders.id = accounts.id) SET status = 'x' RETURNING *;
