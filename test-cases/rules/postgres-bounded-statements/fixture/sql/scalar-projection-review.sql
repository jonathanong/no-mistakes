-- Window calls preserve one value per input row, even when their names are unlisted.
SELECT 1 FROM orders JOIN (SELECT row_number() OVER () AS id) d ON orders.id = d.id;
-- A scalar aggregate keeps one-row cardinality despite unknown scalar calls in its arguments.
SELECT count(custom_scalar(id)) FROM orders;
-- COALESCE rejects set-returning nested calls.
SELECT 1 FROM orders JOIN (SELECT coalesce(custom_scalar($1), $2) AS id) d ON orders.id = d.id;
-- These names are PostgreSQL 18 builtins; on older servers bare calls may resolve to user code.
SELECT 1 FROM orders JOIN (SELECT uuidv7() AS id) d ON orders.id = d.id;
SELECT 1 FROM orders JOIN (SELECT pg_catalog.uuidv7() AS id) d ON orders.id = d.id;
-- Quoting the spelling makes it an ordinary, potentially set-returning function.
SELECT 1 FROM orders JOIN (SELECT "coalesce"($1) AS id) d ON orders.id = d.id;
-- NULLIF, GREATEST and LEAST allow nested SRFs; they are scalar themselves, not traversal boundaries.
SELECT 1 FROM orders JOIN (SELECT nullif(custom_srf($1), $2) AS id) d ON orders.id = d.id;
SELECT 1 FROM orders JOIN (SELECT greatest(custom_srf($1), $2) AS id) d ON orders.id = d.id;
SELECT 1 FROM orders JOIN (SELECT least(custom_srf($1), $2) AS id) d ON orders.id = d.id;
SELECT 1 FROM orders JOIN (SELECT nullif($1, $2) AS id) d ON orders.id = d.id;
SELECT 1 FROM orders JOIN (SELECT greatest($1, $2) AS id) d ON orders.id = d.id;
SELECT 1 FROM orders JOIN (SELECT least($1, $2) AS id) d ON orders.id = d.id;
