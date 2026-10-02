CREATE TABLE unsupported (id int, gen int GENERATED ALWAYS AS (id) VIRTUAL);
-- no-mistakes-disable-next-line postgres-sql-shape-policy
SELECT id FROM orders WHERE id NOT IN (SELECT id FROM bans);
SELECT COUNT(*) > 0 FROM orders;
