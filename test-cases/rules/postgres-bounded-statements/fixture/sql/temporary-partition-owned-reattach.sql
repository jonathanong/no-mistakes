-- The failed second attach cannot transfer ownership away from the first parent.
CREATE TABLE child(id integer);
CREATE TEMP TABLE p1(id integer) PARTITION BY RANGE (id);
CREATE TEMP TABLE p2(id integer) PARTITION BY RANGE (id);
CREATE TEMP TABLE child PARTITION OF p1 FOR VALUES FROM (0) TO (10);
SELECT * FROM child;
DO $$
BEGIN
  ALTER TABLE p2 ATTACH PARTITION child FOR VALUES FROM (0) TO (10);
EXCEPTION WHEN wrong_object_type THEN
  NULL;
END
$$;
DROP TABLE p1;
SELECT * FROM child;
