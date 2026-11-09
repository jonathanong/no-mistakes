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

-- The pinned topic component makes the remaining composite key unique.
SELECT topic_id, COUNT(DISTINCT post_id) /* CASE: recall-composite-count-distinct */
FROM topic_memberships
GROUP BY topic_id;

-- Both the driver key and the slug join key are unique.
SELECT COUNT(DISTINCT r.input_order) /* CASE: recall-one-to-one-join-count */
FROM request_items AS r
JOIN posts AS p ON p.slug = r.slug;

-- The tag join fans out each membership; DISTINCT remains necessary.
SELECT m.topic_id, COUNT(DISTINCT m.post_id) /* CASE: recall-fanout-count-distinct */
FROM topic_memberships AS m
JOIN post_tags AS t ON t.post_id = m.post_id
GROUP BY m.topic_id;

-- Grouping by a primary key makes COUNT(DISTINCT id) redundant.
SELECT id /* CASE: recall-having-distinct-id */
FROM posts
GROUP BY id
HAVING COUNT(DISTINCT id) = 1;

-- The direct aggregate preserves its filter while counting a unique key.
SELECT COUNT(DISTINCT id) FILTER (WHERE author_id IS NOT NULL) /* CASE: recall-filtered-distinct-id */
FROM posts;

-- The outer primary key does not make a scalar subquery's author count unique.
SELECT p.id /* CASE: recall-negative-scalar-author-count */
FROM posts AS p
WHERE (SELECT COUNT(DISTINCT author_id) FROM posts) > 0;
