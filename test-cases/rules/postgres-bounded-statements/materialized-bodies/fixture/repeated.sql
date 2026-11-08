WITH first_pool AS MATERIALIZED (
  SELECT id FROM orders
), second_pool AS MATERIALIZED (
  SELECT id FROM orders
)
SELECT first_pool.id FROM first_pool CROSS JOIN second_pool LIMIT 20;

WITH bounded AS MATERIALIZED (SELECT id FROM orders LIMIT 20),
     unbounded AS MATERIALIZED (SELECT id FROM orders)
SELECT bounded.id FROM bounded CROSS JOIN unbounded LIMIT 20;
