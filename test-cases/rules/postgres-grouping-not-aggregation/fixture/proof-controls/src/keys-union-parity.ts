import { query, sql } from 'db';

query(sql`-- Reverse threshold order is still an all-of candidate count.
SELECT item_id /* CASE: coverage-any-reverse-threshold */
FROM direct_tags WHERE tag_id = ANY($1::uuid[])
GROUP BY item_id HAVING cardinality($1::uuid[]) = COUNT(*)`);

query(sql`-- The distinct key is the requested tag column, not the grouped item key.
SELECT item_id /* CASE: coverage-any-distinct-tag */
FROM direct_tags WHERE tag_id = ANY($1::uuid[])
GROUP BY item_id HAVING COUNT(DISTINCT tag_id) = cardinality($1::uuid[])`);

query(sql`-- Same-argument aggregate ordering does not change the distinct candidate count.
SELECT item_id /* CASE: coverage-any-ordered-distinct-tag */
FROM direct_tags WHERE tag_id = ANY($1::uuid[])
GROUP BY item_id HAVING COUNT(DISTINCT tag_id ORDER BY tag_id) = cardinality($1::uuid[])`);

query(sql`-- Parenthesized nested UNION arms still form one catalogued candidate pool.
SELECT candidate.item_id /* CASE: coverage-nested-union-candidate */
FROM (
  (SELECT item_id, tag_id FROM direct_tags WHERE tag_id = ANY($1::uuid[]))
  UNION
  (SELECT item_id, tag_id FROM alias_tags WHERE tag_id = ANY($1::uuid[])
   UNION SELECT item_id, tag_id FROM direct_tags WHERE tag_id = ANY($1::uuid[]))
) AS candidate
GROUP BY candidate.item_id
HAVING COUNT(DISTINCT candidate.tag_id) = cardinality($1::uuid[])`);

query(sql`-- A union query wrapped in another query remains a candidate pool.
SELECT candidate.item_id /* CASE: coverage-wrapped-union-candidate */
FROM ((SELECT item_id, tag_id FROM direct_tags WHERE tag_id = ANY($1::uuid[])
       UNION SELECT item_id, tag_id FROM alias_tags WHERE tag_id = ANY($1::uuid[]))) AS candidate
GROUP BY candidate.item_id
HAVING COUNT(DISTINCT candidate.tag_id) = cardinality($1::uuid[])`);

query(sql`-- An outer relation fanout invalidates the simple COUNT(*) candidate proof.
SELECT d.item_id /* CASE: coverage-negative-candidate-join-fanout */
FROM direct_tags AS d JOIN post_reads AS r ON r.post_id = d.item_id
WHERE d.tag_id = ANY($1::uuid[])
GROUP BY d.item_id HAVING COUNT(*) = cardinality($1::uuid[])`);

query(sql`-- A second comma-joined relation can multiply candidate rows.
SELECT d.item_id /* CASE: coverage-negative-candidate-comma-source */
FROM direct_tags AS d, post_reads AS r
WHERE d.tag_id = ANY($1::uuid[]) AND r.post_id = d.item_id
GROUP BY d.item_id HAVING COUNT(*) = cardinality($1::uuid[])`);

query(sql`-- Grouping by each source column counts rows, not all matching tags per item.
SELECT * /* CASE: coverage-negative-candidate-wildcard-groups */
FROM direct_tags WHERE tag_id = ANY($1::uuid[])
GROUP BY item_id, tag_id HAVING COUNT(*) = cardinality($1::uuid[])`);

query(sql`-- A threshold unrelated to the requested tag cardinality is not all-of matching.
SELECT item_id /* CASE: coverage-negative-candidate-row-threshold */
FROM direct_tags WHERE tag_id = ANY($1::uuid[])
GROUP BY item_id HAVING COUNT(*) > 0`);

query(sql`-- The all-of equality check remains valid over a limited candidate pool.
SELECT candidate.item_id /* CASE: coverage-limited-candidate-union */
FROM ((SELECT item_id, tag_id FROM direct_tags UNION SELECT item_id, tag_id FROM alias_tags)
      LIMIT 10) AS candidate
GROUP BY candidate.item_id HAVING COUNT(*) = cardinality($1::uuid[])`);

query(sql`-- A subquery-produced array is outside the configured bind-array candidate shape.
SELECT item_id /* CASE: coverage-negative-candidate-subquery-array */
FROM direct_tags
WHERE tag_id = ANY(ARRAY(SELECT post_id FROM post_reads))
GROUP BY item_id HAVING COUNT(*) = cardinality($1::uuid[])`);

query(sql`-- DISTINCT UNION over a VALUES arm preserves its full wildcard projection.
SELECT DISTINCT * /* CASE: coverage-union-values-wildcard */
FROM (VALUES (NULL::uuid) UNION SELECT id FROM current_posts) AS unioned`);

query(sql`-- Aliased output columns are carried from the left UNION arm.
SELECT DISTINCT unioned.subject_id /* CASE: coverage-union-aliased-output */
FROM (
  SELECT topic_id AS subject_id FROM ratings
  UNION SELECT topic_id AS subject_id FROM ratings
) AS unioned`);

query(sql`-- The qualified wildcard must name the derived UNION alias.
SELECT DISTINCT unioned.* /* CASE: coverage-union-qualified-wildcard */
FROM (
  SELECT topic_id, score FROM ratings
  UNION SELECT topic_id, score FROM ratings
) AS unioned`);

query(sql`-- Parenthesized UNION bodies in a CTE retain their output columns.
WITH unioned AS ((SELECT topic_id FROM ratings UNION SELECT topic_id FROM ratings))
SELECT DISTINCT topic_id /* CASE: coverage-union-query-wrapped-cte */ FROM unioned`);

query(sql`-- DISTINCT inside either UNION arm is redundant after UNION's own duplicate removal.
SELECT DISTINCT topic_id /* CASE: coverage-union-arm-left */ FROM ratings
UNION
(SELECT DISTINCT topic_id /* CASE: coverage-union-arm-right */ FROM ratings)`);

query(sql`-- A non-distinct outer projection has no redundant DISTINCT advice.
SELECT topic_id /* CASE: coverage-negative-union-no-outer-distinct */
FROM (SELECT topic_id FROM ratings UNION SELECT topic_id FROM ratings) AS unioned`);

query(sql`-- With no FROM relation, there is no catalogued UNION source to prove.
SELECT DISTINCT 1 /* CASE: coverage-negative-union-no-from */`);

query(sql`-- Multiple FROM items can multiply the selected key.
SELECT DISTINCT current_posts.id /* CASE: coverage-negative-union-many-from */
FROM current_posts, archived_posts`);

query(sql`-- Unknown relation evidence keeps an otherwise UNION-shaped derived table silent.
SELECT DISTINCT unioned.id /* CASE: coverage-negative-union-unknown-relation */
FROM (SELECT id FROM current_posts UNION SELECT id FROM uncatalogued_posts) AS unioned`);

query(sql`-- A recursive CTE wrapper is not treated as a simple UNION catalog projection.
WITH wrapped AS (
  WITH RECURSIVE walk(id, parent_id) AS (
    SELECT id, parent_id FROM posts WHERE id = $1
    UNION ALL SELECT p.id, p.parent_id FROM posts AS p JOIN walk AS w ON p.id = w.parent_id
  )
  SELECT topic_id FROM ratings UNION SELECT topic_id FROM ratings
)
SELECT DISTINCT topic_id /* CASE: coverage-negative-union-recursive-cte */ FROM wrapped`);

query(sql`-- The CTE map must not shadow a schema-qualified physical relation.
WITH current_posts AS (SELECT topic_id FROM ratings UNION SELECT topic_id FROM ratings)
SELECT DISTINCT physical.id /* CASE: coverage-qualified-physical-table */
FROM public.current_posts AS physical`);

query(sql`-- Table functions are not catalogued relations for the UNION projection proof.
SELECT DISTINCT generated.value /* CASE: coverage-negative-union-table-function */
FROM generate_series(1, 3) AS generated(value)`);

query(sql`-- DISTINCT remains useful when a non-unique join side fans the UNION rows out.
SELECT DISTINCT unioned.id /* CASE: coverage-negative-union-join-fanout */
FROM (SELECT id FROM current_posts UNION SELECT id FROM archived_posts) AS unioned
JOIN post_reads AS reads ON reads.post_id = unioned.id`);

query(sql`-- IS NOT NULL predicates supply non-null evidence for a nullable unique key.
SELECT DISTINCT slug /* CASE: coverage-nullable-unique-is-not-null */
FROM nullable_slugs WHERE slug IS NOT NULL`);

query(sql`-- Both the parameter equality and the partial-index predicate are preserved.
SELECT DISTINCT live.topic_id /* CASE: coverage-partial-index-foreign-predicate */
FROM live_topics AS live JOIN posts AS post ON post.id = live.topic_id
WHERE post.id = $1::uuid AND live.deleted_at IS NULL`);

query(sql`-- The nullable unique key is not enough without its partial predicate.
SELECT DISTINCT topic_id /* CASE: coverage-negative-partial-key-unpinned */
FROM live_topics`);

query(sql`-- Nullable unique keys can still hold multiple NULLs for ordinary DISTINCT.
SELECT DISTINCT slug /* CASE: coverage-negative-nullable-unique-null */
FROM nullable_slugs WHERE slug IS NULL`);

query(sql`-- A joined many-side prevents a primary key from proving DISTINCT redundant.
SELECT DISTINCT post.id /* CASE: coverage-negative-unique-join-fanout */
FROM posts AS post JOIN post_reads AS reads ON reads.post_id = post.id`);

query(sql`-- FILTER remains attached while COUNT(DISTINCT primary_key) is redundant.
SELECT COUNT(DISTINCT id) FILTER (WHERE author_id IS NOT NULL) /* CASE: coverage-filtered-count-distinct */
FROM posts`);

query(sql`-- A nested expression still resolves its underlying primary-key column.
SELECT DISTINCT (id) /* CASE: coverage-parenthesized-key */ FROM posts`);

query(sql`-- Repeated grouping sets duplicate even a primary-key row without DISTINCT.
SELECT DISTINCT id /* CASE: coverage-negative-grouping-set */
FROM posts GROUP BY GROUPING SETS ((id), (id))`);

query(sql`-- A derived output alias is recognized when every output is matched by the join.
SELECT post.id
FROM posts AS post
JOIN (SELECT DISTINCT post_id AS target_id /* CASE: coverage-derived-output-alias */ FROM post_reads) AS reads
  ON reads.target_id = post.id`);

query(sql`-- A self-reference to another DISTINCT output means it is not a pure filter join.
SELECT post.id /* CASE: coverage-negative-derived-self-use */
FROM posts AS post
JOIN (SELECT DISTINCT post_id, reader_id FROM post_reads) AS reads
  ON reads.post_id = post.id AND reads.reader_id = reads.reader_id`);

query(sql`-- Catalog proof fails closed when any relation in the UNION is unknown.
SELECT DISTINCT joined.id /* CASE: coverage-negative-catalog-evidence */
FROM (SELECT id FROM current_posts UNION SELECT id FROM uncatalogued_posts) AS joined`);

query(sql`-- A cast parameter still pins the primary key.
SELECT DISTINCT id /* CASE: coverage-cast-pinned-key */
FROM posts WHERE id = $1::uuid`);

query(sql`-- The PK projection stays unique across the grouping-set output rows.
SELECT DISTINCT id /* CASE: coverage-grouping-set-distinct */
FROM posts GROUP BY ROLLUP(id)`);

query(sql`SELECT DISTINCT id /* CASE: coverage-single-grouping-set-key */
FROM posts GROUP BY GROUPING SETS ((id))`);

query(sql`SELECT DISTINCT id /* CASE: coverage-single-cube-key */
FROM posts GROUP BY CUBE(id)`);

query(sql`-- Real aggregate values stay out of the no-op DISTINCT advice.
SELECT DISTINCT author_id, COUNT(*) /* CASE: coverage-negative-real-aggregate */
FROM posts GROUP BY author_id`);

query(sql`-- Ordering within COUNT does not change its DISTINCT value.
SELECT COUNT(DISTINCT id ORDER BY id) /* CASE: coverage-ordered-count-distinct */
FROM posts`);
