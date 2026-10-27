WITH c AS (SELECT id FROM orders) SELECT 1 FROM c JOIN accounts a ON a.id = c.id;
