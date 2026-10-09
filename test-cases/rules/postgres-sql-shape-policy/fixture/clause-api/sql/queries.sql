CREATE TABLE orders (id uuid PRIMARY KEY DEFAULT uuidv7());
ALTER TABLE orders ALTER COLUMN id SET DEFAULT uuidv7();
INSERT INTO orders (id) VALUES (uuidv7());
SELECT uuidv7() AS next_id;
UPDATE orders SET replacement_id = uuidv7() WHERE id = $1;
SELECT id FROM orders WHERE CASE WHEN id >= uuidv7() THEN true ELSE false END; -- finding: scoped
-- The inner predicate overrides the surrounding SELECT list.
SELECT (SELECT count(*) FROM events WHERE id >= uuidv7()) AS recent; -- finding: scoped
SELECT count(*) FROM orders HAVING max(id) >= uuidv7(); -- finding: scoped
SELECT o.id FROM orders o JOIN events e ON e.id >= uuidv7(); -- finding: scoped
SELECT pg_sleep(1); -- finding: legacy
SELECT id FROM orders WHERE id >= uuidv7(); -- no-mistakes-disable-line postgres-sql-shape-policy
-- no-mistakes-disable-next-line postgres-sql-shape-policy
SELECT id FROM orders WHERE id >= uuidv7();
