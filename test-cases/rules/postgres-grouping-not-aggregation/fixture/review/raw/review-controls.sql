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
