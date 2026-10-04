-- A failed attach to an ordinary temp parent does not transfer ownership.
CREATE TABLE child(id integer);
CREATE TEMP TABLE parent(id integer);
CREATE TEMP TABLE child(id integer);
DO $$
BEGIN
  ALTER TABLE parent ATTACH PARTITION child FOR VALUES FROM (0) TO (10);
EXCEPTION WHEN others THEN
  NULL;
END
$$;
DROP TABLE parent;
SELECT * FROM child;
-- An explicit transaction does not turn an ordinary parent into a partitioned one.
BEGIN;
CREATE TEMP TABLE other_parent(id integer);
ALTER TABLE child RENAME TO renamed_child;
COMMIT;
DROP TABLE other_parent;
SELECT * FROM renamed_child;
-- DETACH from the wrong partitioned parent also leaves the original link.
DROP TABLE renamed_child;
CREATE TEMP TABLE p1(id integer) PARTITION BY RANGE (id);
CREATE TEMP TABLE p2(id integer) PARTITION BY RANGE (id);
CREATE TEMP TABLE child PARTITION OF p1 FOR VALUES FROM (0) TO (10);
DO $$
BEGIN
  ALTER TABLE p2 DETACH PARTITION child;
EXCEPTION WHEN others THEN
  NULL;
END
$$;
DROP TABLE p1;
SELECT * FROM child;
