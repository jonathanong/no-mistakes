-- cte-shadows-physical-primary-key
WITH posts AS (SELECT post_id AS id FROM post_reads)
SELECT DISTINCT id FROM posts
;

-- grouped-jsonb-aggregate
SELECT id, jsonb_agg(post_id) FROM post_reads GROUP BY id
;

-- grouped-user-aggregate
SELECT id, review_custom_aggregate(post_id) FROM post_reads GROUP BY id
;

-- any-in-unrelated-existence-filter
SELECT topic_id FROM ratings
WHERE EXISTS (SELECT 1 FROM posts WHERE id = ANY($1::uuid[]))
GROUP BY topic_id HAVING COUNT(*) = $2
;

-- unqualified-derived-projection
SELECT post_id FROM posts AS p
JOIN (SELECT DISTINCT post_id FROM post_reads) AS reads
ON reads.post_id = p.id
;

-- unqualified-derived-order
SELECT p.id FROM posts AS p
JOIN (SELECT DISTINCT post_id FROM post_reads) AS reads
ON reads.post_id = p.id ORDER BY post_id
;

-- filter-side-aggregate-in-order
SELECT COUNT(DISTINCT p.id) FROM posts AS p
JOIN post_tags AS t ON t.post_id = p.id ORDER BY COUNT(t.id)
;

-- quoted-cte-shadows-physical-primary-key
WITH "posts" AS (SELECT post_id AS id FROM post_reads)
SELECT DISTINCT id FROM "posts"
;

-- uppercase-cte-shadows-physical-primary-key
WITH POSTS AS (SELECT post_id AS id FROM post_reads)
SELECT DISTINCT id FROM POSTS
;

-- custom-aggregate-is-a-distinct-cte-consumer
WITH watched AS (SELECT DISTINCT post_id FROM post_reads)
SELECT p.id FROM posts AS p
WHERE p.id IN (SELECT review_custom_aggregate(post_id) FROM watched)
;

-- bare-absence-needs-null-threshold-guard
SELECT r.post_id FROM post_reads AS r
GROUP BY r.post_id
HAVING NOT EXISTS (
  SELECT 1 FROM post_reads AS older WHERE older.post_id = r.post_id
) OR MAX(r.created_at) < $1
;

-- bare-absence-needs-original-null-key-disjunct
SELECT p.parent_id FROM posts AS p
GROUP BY p.parent_id
HAVING NOT EXISTS (
  SELECT 1 FROM posts AS older WHERE older.parent_id = p.parent_id
) OR MAX(p.created_at) < $1
;

-- CTE DISTINCT also determines a correlated scalar count used by ORDER BY.
WITH watched AS (SELECT DISTINCT post_id FROM post_reads)
SELECT p.id FROM posts AS p
WHERE p.id IN (SELECT post_id FROM watched)
ORDER BY (SELECT COUNT(*) FROM watched AS w WHERE w.post_id = p.id);

-- COUNT(DISTINCT item_id) does not count the requested tag matches.
SELECT item_id FROM direct_tags
WHERE tag_id = ANY($1::uuid[])
GROUP BY item_id
HAVING COUNT(DISTINCT item_id) = cardinality($1::uuid[]);

-- Counting a bind value does not count distinct pool terms.
SELECT p.item_id FROM (
  SELECT item_id, tag_id FROM direct_tags
  UNION SELECT item_id, tag_id FROM alias_tags
) p GROUP BY p.item_id HAVING COUNT(DISTINCT $3) = $2;

-- nullable-max-needs-nonnull-input-witness
SELECT t.topic_id FROM live_topics AS t GROUP BY t.topic_id
HAVING NOT EXISTS (SELECT 1 FROM live_topics AS old WHERE old.topic_id = t.topic_id AND old.deleted_at IS NOT NULL)
OR MAX(t.deleted_at) < $1;

-- unknown-max-column-is-not-catalog-evidence
SELECT t.topic_id FROM live_topics AS t GROUP BY t.topic_id
HAVING NOT EXISTS (SELECT 1 FROM live_topics AS old WHERE old.topic_id = t.topic_id)
OR MAX(t.missing_timestamp) < $1;
