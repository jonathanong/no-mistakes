-- all-of-candidate-pool: the candidate relation must be filtered as one predicate per requested tag.
SELECT candidate.item_id /* CASE: raw-all-of-distinct */
FROM (
  SELECT item_id, tag_id FROM direct_tags WHERE tag_id = ANY($1::uuid[])
  UNION
  SELECT item_id, tag_id FROM alias_tags WHERE tag_id = ANY($1::uuid[])
) AS candidate
GROUP BY candidate.item_id
HAVING COUNT(DISTINCT candidate.tag_id) = cardinality($1::uuid[]);

-- all-of-candidate-pool: a numeric threshold on a simple ANY candidate pool.
SELECT item_id /* CASE: raw-all-of-count-star */
FROM direct_tags
WHERE tag_id = ANY($1::uuid[])
GROUP BY item_id
HAVING COUNT(*) = $2;

-- distinct-over-unique-key: topic_id is fixed and post_id completes the composite primary key.
SELECT DISTINCT post_id /* CASE: raw-unique-composite-distinct */
FROM topic_memberships
WHERE topic_id = $1;

-- distinct-over-unique-key: GROUP BY a remaining PK component after pinning the other one.
SELECT post_id /* CASE: raw-unique-composite-group */
FROM topic_memberships
WHERE topic_id = $1
GROUP BY post_id;

-- distinct-over-unique-key: DISTINCT over the primary key itself is a no-op.
SELECT DISTINCT id /* CASE: raw-unique-primary-key */ FROM posts;

-- distinct-over-unique-key: counting distinct primary keys is a no-op.
SELECT COUNT(DISTINCT id) /* CASE: raw-unique-count-distinct */ FROM posts;

-- distinct-over-unique-key: exact predicate implication for a partial unique index.
SELECT DISTINCT topic_id /* CASE: raw-unique-partial */
FROM live_topics
WHERE deleted_at IS NULL;

-- negative control: an OR arm does not imply the partial unique-index predicate.
SELECT DISTINCT topic_id /* CASE: raw-negative-partial-or */
FROM live_topics
WHERE deleted_at IS NULL OR topic_id = $1;

-- negative control: a partial unique index does not cover rows without its predicate.
SELECT DISTINCT topic_id /* CASE: raw-negative-partial-missing */
FROM live_topics;

-- negative control: NULLS DISTINCT does not make duplicate NULL slug values unique.
SELECT DISTINCT slug /* CASE: raw-negative-null */
FROM nullable_slugs
WHERE slug IS NULL;

-- negative control: nullable unique keys may contain multiple NULLs.
SELECT DISTINCT slug /* CASE: raw-negative-null-unfiltered */ FROM nullable_slugs;

-- distinct-over-unique-key: both the driver key and joined slug are unique.
SELECT DISTINCT ON (r.input_order) p.id /* CASE: raw-unique-distinct-on-join */
FROM request_items AS r
JOIN posts AS p ON p.slug = r.slug
ORDER BY r.input_order;

-- distinct-over-union: UNION already removes duplicate projected ids.
SELECT DISTINCT id /* CASE: raw-union-derived */
FROM (
  SELECT id FROM current_posts
  UNION
  SELECT id FROM archived_posts
) AS both_posts;

-- distinct-over-union: DISTINCT inside a UNION arm is redundant as well.
WITH both_posts AS (
  SELECT DISTINCT id /* CASE: raw-union-arm-left */ FROM current_posts
  UNION
  SELECT DISTINCT id /* CASE: raw-union-arm-right */ FROM archived_posts
)
SELECT id FROM both_posts;

-- distinct-over-union: outer DISTINCT over a CTE that exposes the same UNION projection.
WITH both_posts AS (
  SELECT id FROM current_posts
  UNION
  SELECT id FROM archived_posts
)
SELECT DISTINCT id /* CASE: raw-union-cte */ FROM both_posts;

-- distinct-join-filter: duplicate read rows are only used to test matching posts.
SELECT p.id /* CASE: raw-filter-derived-join */
FROM posts AS p
JOIN (SELECT DISTINCT post_id FROM post_reads WHERE reader_id = $1) AS reads
  ON reads.post_id = p.id;

-- negative control: DISTINCT covers both derived keys but the join predicate uses only one.
SELECT p.id /* CASE: raw-negative-partial-join-key */
FROM posts AS p
JOIN (SELECT DISTINCT post_id, reader_id FROM post_reads) AS reads
  ON reads.post_id = p.id;

-- negative control: a later JOIN ON clause reads the derived DISTINCT alias.
SELECT p.id /* CASE: raw-negative-derived-later-join */
FROM posts AS p
JOIN (SELECT DISTINCT post_id, reader_id FROM post_reads) AS reads
  ON reads.post_id = p.id
JOIN posts AS reader ON reader.author_id = reads.reader_id;

-- negative control: parent ORDER BY reads a column from the derived DISTINCT alias.
SELECT p.id /* CASE: raw-negative-derived-order-by */
FROM posts AS p
JOIN (SELECT DISTINCT post_id, created_at FROM post_reads) AS reads
  ON reads.post_id = p.id
ORDER BY reads.created_at;

-- distinct-join-filter: duplicates do not affect ALL membership semantics.
SELECT p.id /* CASE: raw-filter-all-membership */
FROM posts AS p
WHERE p.id <> ALL(ARRAY(SELECT DISTINCT post_id FROM blocked_posts WHERE active));

-- distinct-join-filter: every consumer uses the distinct CTE for membership only.
WITH watched AS (
  SELECT DISTINCT post_id FROM post_reads WHERE reader_id = $1
)
SELECT p.id /* CASE: raw-filter-cte-in */ FROM posts AS p WHERE p.id IN (SELECT post_id FROM watched);

-- distinct-join-filter: every consumer uses the distinct CTE for anti-membership only.
WITH watched AS (
  SELECT DISTINCT post_id FROM post_reads WHERE reader_id = $1
)
SELECT p.id /* CASE: raw-filter-cte-not-exists */ FROM posts AS p
WHERE NOT EXISTS (SELECT 1 FROM watched AS w WHERE w.post_id = p.id);

-- distinct-join-filter: DISTINCT masks join fan-out used only as a filter.
SELECT p.author_id, COUNT(DISTINCT p.id) AS post_count /* CASE: raw-filter-count-distinct */
FROM posts AS p
JOIN post_tags AS t ON t.post_id = p.id
WHERE t.tag_id = ANY($1::uuid[])
GROUP BY p.author_id;

-- aggregate-existence-test: scalar row count is used only as a boolean.
SELECT p.id /* CASE: raw-count-positive */ FROM posts AS p
WHERE (SELECT COUNT(*) FROM post_reads AS r WHERE r.post_id = p.id) > 0;

-- aggregate-existence-test: zero matching rows is an anti-existence check.
SELECT p.id /* CASE: raw-count-zero */ FROM posts AS p
WHERE (SELECT COUNT(*) FROM post_reads AS r WHERE r.post_id = p.id) = 0;

-- aggregate-existence-test: direct scalar COUNT results are boolean tests.
SELECT COUNT(*) > 0 /* CASE: raw-count-direct-positive */ FROM post_reads WHERE reader_id = $1;
SELECT COUNT(*) = 0 /* CASE: raw-count-direct-zero */ FROM post_reads WHERE reader_id = $1;

-- aggregate-existence-test: the OR and MAX threshold express absence of a qualifying row.
SELECT r.post_id /* CASE: raw-aggregate-not-exists-max */
FROM post_reads AS r
GROUP BY r.post_id
HAVING NOT EXISTS (
  SELECT 1 FROM post_reads AS newer
  WHERE newer.post_id = r.post_id AND newer.created_at >= $1
) OR MAX(r.created_at) < $1;

-- negative control: actual aggregate values are projected.
SELECT topic_id, COUNT(*) AS rating_count, SUM(score) AS total_score /* CASE: raw-negative-real-aggregate */
FROM ratings
GROUP BY topic_id;

-- negative control: grouping by a non-unique key to report aggregate values.
SELECT author_id, COUNT(*) AS post_count, MAX(created_at) AS latest_post /* CASE: raw-negative-nonunique-aggregate */
FROM posts
GROUP BY author_id;

-- negative control: DISTINCT removes the same post returned by the id and slug lookups.
WITH requested_posts AS (
  SELECT id, 'id'::text AS source FROM posts WHERE id = $1
  UNION
  SELECT id, 'slug'::text AS source FROM posts WHERE slug = $2
)
SELECT DISTINCT id /* CASE: raw-negative-requested-union */ FROM requested_posts;

-- negative control: project the joined side, so DISTINCT protects its distinct values.
SELECT DISTINCT p.id, reads.created_at /* CASE: raw-negative-projected-join-side */
FROM posts AS p
JOIN (SELECT DISTINCT post_id, created_at FROM post_reads) AS reads ON reads.post_id = p.id;

-- negative control: LIMIT/OFFSET makes membership depend on the distinct candidate rows.
SELECT p.id /* CASE: raw-negative-limited-membership */
FROM posts AS p
WHERE p.id IN (
  SELECT DISTINCT post_id FROM post_reads ORDER BY post_id LIMIT 10 OFFSET 2
);

-- negative control: an outer join to non-unique reads multiplies a unique post row.
SELECT DISTINCT p.id /* CASE: raw-negative-outer-join-unique-driver */
FROM posts AS p
LEFT JOIN post_reads AS reads ON reads.post_id = p.id;

-- negative control: one non-membership consumer makes the CTE's DISTINCT observable.
WITH watched AS (SELECT DISTINCT post_id FROM post_reads)
SELECT p.id FROM posts AS p WHERE p.id IN (SELECT post_id FROM watched)
UNION ALL
SELECT watched.post_id FROM watched JOIN posts AS p ON p.id = watched.post_id;

-- negative control: UNION ALL preserves the source rows and DISTINCT is semantically needed.
SELECT DISTINCT id /* CASE: raw-negative-union-all-distinct */
FROM (
  SELECT id FROM current_posts
  UNION ALL
  SELECT id FROM archived_posts
) AS all_posts;

-- negative control: UNION ALL projects each source row and preserves its source tag.
SELECT id, 'current'::text AS source /* CASE: raw-negative-union-all-projection */ FROM current_posts
UNION ALL
SELECT id, 'archived'::text AS source FROM archived_posts;

-- negative control: DISTINCT ON chooses the newest row per non-unique author.
SELECT DISTINCT ON (author_id) id, author_id, created_at /* CASE: raw-negative-latest-distinct-on */
FROM posts
ORDER BY author_id, created_at DESC;

-- negative control: DISTINCT is a recursive cycle guard.
WITH RECURSIVE ancestors(id, parent_id) AS (
  SELECT id, parent_id FROM posts WHERE id = $1
  UNION ALL
  SELECT p.id, p.parent_id FROM posts AS p
  JOIN ancestors AS a ON p.id = a.parent_id
)
SELECT DISTINCT id /* CASE: raw-negative-recursive */ FROM ancestors;

-- negative control: the multiplied side is projected as a real aggregate.
SELECT p.author_id, COUNT(DISTINCT p.id), MAX(t.created_at) /* CASE: raw-negative-projected-aggregate */
FROM posts AS p
JOIN post_tags AS t ON t.post_id = p.id
GROUP BY p.author_id;

-- negative control: FILTER and a window produce values rather than test existence.
SELECT author_id,
  COUNT(*) FILTER (WHERE created_at >= $1) OVER (PARTITION BY author_id) AS recent_count,
  COUNT(*) OVER (PARTITION BY author_id) AS all_count /* CASE: raw-negative-filter-window */
FROM posts;
