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
-- Both conflict actions keep the partial-index arbiter's WHERE context.
INSERT INTO orders(id) VALUES (1) ON CONFLICT (id) WHERE uuidv7() IS NOT NULL DO NOTHING; -- finding: scoped
INSERT INTO orders(id) VALUES (2) ON CONFLICT (id) WHERE uuidv7() IS NOT NULL DO UPDATE SET id = 2; -- finding: scoped
INSERT INTO orders(id) VALUES (3) ON CONFLICT ((uuidv7())) DO NOTHING;
SELECT 1 WHERE "custom.schema"."odd.function"() IS NOT NULL; -- finding: quoted
-- Window partitions/frames are not SELECT lists.
SELECT sum(1) OVER (PARTITION BY probe_boundary() ROWS probe_boundary() PRECEDING);
SELECT sum(1) OVER w FROM orders WINDOW w AS (PARTITION BY probe_boundary() ROWS probe_boundary() PRECEDING);
SELECT probe_boundary(); -- finding: boundary

CREATE INDEX orders_partial_idx ON orders (id) WHERE id >= uuidv7(INTERVAL '-30 days'); -- finding: scoped
CREATE INDEX orders_expression_idx ON orders (probe_boundary(id)); -- permitted: index expressions are unscoped
