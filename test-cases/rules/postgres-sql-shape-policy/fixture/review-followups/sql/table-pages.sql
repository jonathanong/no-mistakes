-- TABLE is a single-relation whole-table page, with the same scope/cap guards as SELECT.
TABLE orders ORDER BY id LIMIT $1;
TABLE public.orders ORDER BY orders.id LIMIT $1;
TABLE "Order Items" ORDER BY id LIMIT 5;
(TABLE orders) ORDER BY id LIMIT $1;
WITH q AS (SELECT * FROM orders) (TABLE q) ORDER BY id LIMIT $1; -- Parentheses retain the supported CTE/TABLE grammar.
TABLE orders ORDER BY id LIMIT 0;
TABLE orders ORDER BY random() LIMIT $1;
TABLE orders ORDER BY id LIMIT NULL;
TABLE orders
ORDER BY id
LIMIT $1;
-- no-mistakes-disable-next-line postgres-sql-shape-policy
TABLE orders ORDER BY id LIMIT $1;
