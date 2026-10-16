WITH changed AS (
  INSERT INTO t(id) VALUES (1) RETURNING id + 1
)
SELECT * FROM changed;
WITH changed AS (
  INSERT INTO t(id) VALUES (1) RETURNING id BETWEEN 1 AND 2
)
SELECT * FROM changed;
