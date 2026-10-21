-- PostgreSQL only accepts a predicate after a parenthesised column target.
WITH constrained AS (
 INSERT INTO target (id) VALUES (1)
 ON CONFLICT ON CONSTRAINT target_pkey WHERE id > 0 DO NOTHING
)
SELECT id FROM constrained;
SELECT 42 AS kept;

WITH bare AS (
 INSERT INTO target (id) VALUES (1)
 ON CONFLICT WHERE id > 0 DO NOTHING
)
SELECT id FROM bare;
SELECT 7 AS kept;
