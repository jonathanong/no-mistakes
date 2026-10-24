-- Placeholder indexes cannot prove a match; NULLS NOT DISTINCT accepts IS NULL, plain UNIQUE does not.
SELECT 1 FROM placeholder_partial AS t WHERE t.email = $2 AND t.state = $1;
SELECT 1 FROM placeholder_expression AS t WHERE lower(t.email) || $1 = $2;
-- NULLS NOT DISTINCT accepts its single NULL index entry; plain UNIQUE permits duplicates.
SELECT 1 FROM nulls_nd AS t WHERE t.email IS NULL;
SELECT 1 FROM nulls_plain AS t WHERE t.email IS NULL;
