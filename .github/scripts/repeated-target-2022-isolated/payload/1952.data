INSERT INTO t(id) SELECT now /*keep*/ () RETURNING id;
WITH kept AS (
  INSERT INTO t(id) SELECT now /*keep*/ () RETURNING id
)
SELECT id FROM kept;
WITH plain AS (
  INSERT INTO t(id) SELECT now /*keep*/ ()
)
SELECT 1;
