SELECT * FROM orders LIMIT (SELECT count(*) FROM orders);
