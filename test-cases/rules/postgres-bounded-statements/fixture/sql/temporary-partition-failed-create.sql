-- CREATE PARTITION OF an ordinary parent fails before the child is created.
CREATE TABLE child(id integer);
CREATE TEMP TABLE parent(id integer);
DO $$
BEGIN
  CREATE TEMP TABLE child PARTITION OF parent FOR VALUES FROM (0) TO (10);
EXCEPTION WHEN others THEN
  NULL;
END
$$;
DROP TABLE parent;
SELECT * FROM child;
