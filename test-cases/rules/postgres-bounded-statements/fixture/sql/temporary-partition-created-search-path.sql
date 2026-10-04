-- CREATE TEMP fixes the child identity even when public precedes pg_temp.
CREATE TABLE child(id integer);
SET search_path = public, pg_temp;
CREATE TEMP TABLE parent(id integer) PARTITION BY RANGE (id);
CREATE TEMP TABLE child PARTITION OF pg_temp.parent FOR VALUES FROM (0) TO (10);
SELECT * FROM pg_temp.child;
DROP TABLE pg_temp.parent;
RESET search_path;
SELECT * FROM child;
