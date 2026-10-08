-- WITH INSERT normalization must reach the same AST child beneath each wrapper.
WITH src AS (SELECT 1 AS id)
INSERT INTO target SELECT id FROM src ON CONFLICT (id) WHERE id > 0 DO NOTHING;
EXPLAIN ANALYZE WITH src AS (SELECT 1 AS id)
INSERT INTO target SELECT id FROM src ON CONFLICT (id) WHERE id > 0 DO NOTHING;
PREPARE with_insert AS WITH src AS (SELECT 1 AS id)
INSERT INTO target SELECT id FROM src ON CONFLICT (id) WHERE id > 0 DO NOTHING;
CREATE FUNCTION with_insert_body() RETURNS void BEGIN ATOMIC
  WITH src AS (SELECT 1 AS id)
  INSERT INTO target SELECT id FROM src ON CONFLICT (id) WHERE id > 0 DO NOTHING;
END;
-- VALUES has no query-fact slot for the outer WITH: retain an explicit incomplete child.
EXPLAIN WITH src AS (SELECT 1 AS id)
INSERT INTO target VALUES (1) ON CONFLICT (id) WHERE id > 0 DO NOTHING;
-- A malformed target predicate must preserve the following independent query.
EXPLAIN WITH src AS (SELECT 1 AS id)
INSERT INTO target SELECT id FROM src ON CONFLICT (id) WHERE DO NOTHING;
SELECT 71;
PREPARE bad_with AS WITH src AS (SELECT 1 AS id)
INSERT INTO target SELECT id FROM src ON CONFLICT (id) WHERE id > 0 DO UPDATE SET id = ;
SELECT 72;
