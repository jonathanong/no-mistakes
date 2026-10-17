CREATE TRIGGER touch AFTER INSERT ON orders FOR EACH ROW EXECUTE FUNCTION touch();
INSERT INTO orders (id) VALUES ($1) RETURNING *;
SELECT o.id FROM orders o WHERE EXISTS (
 SELECT 1 FROM events e WHERE e.order_id = o.id
 UNION ALL SELECT 1 FROM archived e WHERE e.order_id = o.id
);
UPDATE orders SET status = 'done' WHERE created_at = now();
