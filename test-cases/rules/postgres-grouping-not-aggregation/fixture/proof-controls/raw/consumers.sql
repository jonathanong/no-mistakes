WITH watched AS (SELECT DISTINCT post_id FROM post_reads) -- CASE exists-star
SELECT p.id FROM posts p WHERE EXISTS (SELECT * FROM watched w WHERE w.post_id = p.id);

WITH watched AS (SELECT DISTINCT post_id FROM post_reads) -- CASE not-exists-qualified-star
SELECT p.id FROM posts p WHERE NOT EXISTS (SELECT w.* FROM watched w WHERE w.post_id = p.id);

WITH watched AS (SELECT DISTINCT post_id FROM post_reads) -- CASE membership-column
SELECT p.id FROM posts p WHERE p.id IN (SELECT post_id FROM watched);

WITH watched AS (SELECT DISTINCT post_id FROM post_reads) -- CASE count-two-membership
SELECT p.id FROM posts p WHERE 2 IN (SELECT COUNT(*) FROM watched);

WITH watched AS (SELECT DISTINCT post_id FROM post_reads) -- CASE offset-changes-membership
SELECT p.id FROM posts p WHERE p.id IN (SELECT post_id FROM watched OFFSET 1);

WITH watched AS (SELECT DISTINCT post_id FROM post_reads) -- CASE having-count-retains-cardinality
SELECT p.id FROM posts p WHERE EXISTS (SELECT post_id FROM watched GROUP BY post_id HAVING COUNT(*) > 1);

WITH watched AS (SELECT DISTINCT post_id FROM post_reads) -- CASE joined-count-retains-cardinality
SELECT p.id FROM posts p WHERE 2 IN (SELECT COUNT(*) FROM watched w JOIN post_reads r ON r.post_id = w.post_id);
