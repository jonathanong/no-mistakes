SELECT * FROM orders();
SELECT ARRAY[row_to_json(o.*)] FROM orders o;
SELECT array_agg(id ORDER BY row_to_json(o.*)) FROM orders o;
