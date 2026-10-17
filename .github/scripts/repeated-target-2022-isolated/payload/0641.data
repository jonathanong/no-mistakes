-- Source-level WITH is an unsupported VALUES modifier. The row stays mapped.
INSERT INTO t(id)
WITH seed AS (SELECT 1)
VALUES (1)
RETURNING id;

-- The CTE form must map that same VALUES row, not unsupportedSource.
WITH changed AS (
  INSERT INTO t(id)
  WITH seed AS (SELECT 1)
  VALUES (1)
  RETURNING id
)
SELECT * FROM changed;

-- Non-RETURNING CTE inserts still omit columnSources. The seed CTE stays recorded.
WITH plain AS (
  INSERT INTO t(id)
  WITH seed AS (SELECT 1)
  VALUES (1)
)
SELECT 1;

-- SELECT sources record this same source-level seed on the source query scope.
WITH changed AS (
  INSERT INTO t(id)
  WITH seed AS (SELECT 1)
  SELECT 1
  RETURNING id
)
SELECT * FROM changed;
