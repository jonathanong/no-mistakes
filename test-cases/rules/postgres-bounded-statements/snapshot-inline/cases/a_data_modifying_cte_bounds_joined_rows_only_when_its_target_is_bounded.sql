WITH moved AS (INSERT INTO archive SELECT * FROM orders RETURNING id) DELETE FROM orders USING moved WHERE orders.id = moved.id;
