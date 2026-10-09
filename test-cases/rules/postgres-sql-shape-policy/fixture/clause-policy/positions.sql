-- Every numbered clause is intentional; nested expressions inherit it.
SELECT uuidv7() FROM orders WHERE uuidv7() IS NOT NULL GROUP BY id HAVING uuidv7() IS NOT NULL ORDER BY uuidv7();
SELECT uuidv7() FROM orders o JOIN events e ON e.id >= uuidv7();
INSERT INTO orders (id) VALUES (uuidv7()) ON CONFLICT (id) DO UPDATE SET id = uuidv7() WHERE id >= uuidv7() RETURNING uuidv7();
UPDATE orders SET id = uuidv7() WHERE id >= uuidv7() RETURNING uuidv7();
DELETE FROM orders WHERE id >= uuidv7() RETURNING uuidv7();
MERGE INTO orders o USING events e ON o.id >= uuidv7() WHEN MATCHED AND e.id >= uuidv7() THEN UPDATE SET id = uuidv7() WHEN NOT MATCHED THEN INSERT (id) VALUES (uuidv7());
CREATE TABLE generated_ids (id uuid DEFAULT uuidv7());
ALTER TABLE generated_ids ALTER COLUMN id SET DEFAULT uuidv7();
SELECT (SELECT uuidv7() WHERE uuidv7() IS NOT NULL) WHERE min_id_for(uuidv7()) IS NOT NULL;
CREATE VIEW recent_orders AS SELECT id FROM orders WHERE id >= uuidv7();
WITH recent AS (SELECT id FROM orders WHERE id >= uuidv7()) SELECT uuidv7() FROM recent;
SELECT id FROM orders WHERE CASE WHEN id >= uuidv7() THEN true ELSE false END;
SELECT pg_catalog.uuidv7() WHERE pg_catalog.uuidv7() IS NOT NULL;
MERGE INTO orders o USING events e ON o.id = e.id WHEN MATCHED THEN DELETE RETURNING uuidv7();
