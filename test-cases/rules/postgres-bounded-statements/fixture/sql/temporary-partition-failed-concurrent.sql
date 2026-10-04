-- CONCURRENTLY fails inside DO; the child still belongs to its parent.
CREATE TABLE child(id integer);
CREATE TEMP TABLE parent(id integer) PARTITION BY RANGE (id);
CREATE TEMP TABLE child PARTITION OF parent FOR VALUES FROM (0) TO (10);
DO $$
BEGIN
  ALTER TABLE parent DETACH PARTITION child CONCURRENTLY;
EXCEPTION WHEN others THEN
  NULL;
END
$$;
DROP TABLE parent;
SELECT * FROM child;
-- COMMIT AND CHAIN keeps a transaction open for the next statement.
CREATE TEMP TABLE parent(id integer) PARTITION BY RANGE (id);
CREATE TEMP TABLE child PARTITION OF parent FOR VALUES FROM (0) TO (10);
BEGIN;
COMMIT AND CHAIN;
SAVEPOINT before_chained_detach;
ALTER TABLE parent DETACH PARTITION child CONCURRENTLY;
ROLLBACK TO SAVEPOINT before_chained_detach;
COMMIT;
DROP TABLE parent;
SELECT * FROM child;
-- The same modifier also fails inside an explicit transaction.
CREATE TEMP TABLE parent(id integer) PARTITION BY RANGE (id);
CREATE TEMP TABLE child PARTITION OF parent FOR VALUES FROM (0) TO (10);
BEGIN;
SAVEPOINT before_detach;
ALTER TABLE parent DETACH PARTITION child CONCURRENTLY;
ROLLBACK TO SAVEPOINT before_detach;
COMMIT;
DROP TABLE parent;
SELECT * FROM child;
