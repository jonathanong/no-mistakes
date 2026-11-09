-- COUNT ignores NULLs, so repeated NULL slugs do not make this aggregate useful.
SELECT COUNT(DISTINCT slug) /* CASE: recall-nullable-count-distinct */ FROM nullable_slugs;

-- DISTINCT over UNION covers the whole wildcard projection.
SELECT DISTINCT * /* CASE: recall-union-star */ FROM (
  SELECT * FROM current_posts
  UNION
  SELECT * FROM archived_posts
) AS all_posts;

-- The UNION removes duplicate ids before the outer query adds its deterministic expression.
SELECT DISTINCT id, id::text AS display_id /* CASE: recall-union-extra-expression */
FROM (
  SELECT id FROM current_posts
  UNION
  SELECT id FROM archived_posts
) AS all_posts;
