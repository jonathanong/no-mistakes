INSERT INTO orders AS o (id) VALUES (1) RETURNING o.*;
INSERT INTO orders DEFAULT VALUES RETURNING *;
SELECT public.orders.* FROM public.orders;
SELECT row_to_json(row => o.*) FROM orders o;
SELECT row_to_json(row := o.*) FROM orders o;
