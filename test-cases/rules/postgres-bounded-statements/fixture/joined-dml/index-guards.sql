-- These malformed catalog indexes cannot prove a unique match.
SELECT 1 FROM placeholder_partial AS t WHERE t.email = $1 AND t.state = $2;
SELECT 1 FROM placeholder_expression AS t WHERE lower(t.email) || $1 = $2;
-- NULLS NOT DISTINCT treats the single NULL index entry as unique; ordinary UNIQUE does not.
SELECT 1 FROM nulls_nd AS t WHERE t.email IS NULL;
SELECT 1 FROM nulls_plain AS t WHERE t.email IS NULL;
