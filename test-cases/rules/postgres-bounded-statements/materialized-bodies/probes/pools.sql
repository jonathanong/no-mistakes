WITH a AS MATERIALIZED (
  SELECT DISTINCT user_id FROM source_rows WHERE group_id = $1 AND active
), b AS MATERIALIZED (
  SELECT a.user_id, m.id FROM a JOIN LATERAL (
    SELECT id FROM member_rows WHERE user_id = a.user_id LIMIT 1
  ) m ON true
) SELECT user_id FROM b LIMIT 1;

SELECT id FROM lookup_rows WHERE id IN (SELECT user_id FROM edge_rows) LIMIT 1;
SELECT id FROM (SELECT id FROM edge_rows) pool LIMIT 1;

WITH a AS MATERIALIZED (
  SELECT user_id FROM source_rows WHERE group_id = $1
), b AS MATERIALIZED (
  SELECT a.user_id, e.id FROM a CROSS JOIN edge_rows e
) SELECT user_id FROM b LIMIT 1;

WITH original AS (SELECT user_id FROM source_rows),
a AS MATERIALIZED (SELECT user_id FROM original),
b AS MATERIALIZED (SELECT user_id FROM original)
SELECT a.user_id FROM a JOIN b ON a.user_id = b.user_id LIMIT 1;

WITH a AS MATERIALIZED (SELECT id FROM source_rows),
b AS MATERIALIZED (
  WITH a AS (SELECT id FROM edge_rows) SELECT id FROM a
) SELECT id FROM b LIMIT 1;

WITH a AS MATERIALIZED (SELECT id FROM source_rows),
b AS MATERIALIZED (TABLE a)
SELECT id FROM b LIMIT 1;

WITH a AS MATERIALIZED (SELECT id FROM source_rows),
b AS MATERIALIZED (
  SELECT id FROM lookup_rows WHERE id IN (SELECT id FROM a)
) SELECT id FROM b LIMIT 1;

WITH a AS MATERIALIZED (SELECT id FROM source_rows),
b AS MATERIALIZED (
  SELECT k.id FROM lookup_rows k WHERE k.id = ANY(ARRAY(SELECT a.id FROM a))
) SELECT id FROM b LIMIT 1;
