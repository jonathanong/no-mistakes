WITH posts AS (SELECT DISTINCT id /* CASE: order-self-physical-key */ FROM posts) SELECT id FROM posts;

WITH early AS (SELECT DISTINCT id /* CASE: order-forward-physical-key */ FROM posts), posts(id) AS (VALUES(NULL::uuid)) SELECT * FROM early;

WITH posts(id) AS (VALUES(NULL::uuid)), early AS (SELECT DISTINCT id /* CASE: order-negative-prior-shadow */ FROM posts) SELECT * FROM early;

WITH early AS (SELECT DISTINCT topic_id /* CASE: order-negative-future-union-lineage */ FROM ratings), ratings AS (SELECT topic_id FROM topic_memberships UNION SELECT topic_id FROM live_topics) SELECT * FROM early;

WITH ratings AS (SELECT topic_id FROM topic_memberships UNION SELECT topic_id FROM live_topics), early AS (SELECT DISTINCT topic_id /* CASE: order-prior-union-lineage */ FROM ratings) SELECT * FROM early;

SELECT DISTINCT id /* CASE: order-evidence-self-physical */ FROM (WITH posts AS (SELECT id FROM posts) SELECT id FROM current_posts UNION SELECT id FROM archived_posts) pool;

SELECT DISTINCT id /* CASE: order-evidence-forward-physical */ FROM (WITH earlier AS (SELECT id FROM posts), posts(id) AS (VALUES(NULL::uuid)) SELECT id FROM current_posts UNION SELECT id FROM archived_posts) pool;
WITH post_reads AS (SELECT DISTINCT post_id /* CASE: order-self-physical-exists */ FROM post_reads) SELECT p.id FROM posts p WHERE EXISTS(SELECT * FROM post_reads r WHERE r.post_id=p.id);

WITH post_reads AS (SELECT DISTINCT post_id /* CASE: order-self-physical-in */ FROM post_reads) SELECT p.id FROM posts p WHERE p.id IN(SELECT post_id FROM post_reads);

WITH post_reads AS (SELECT DISTINCT post_id /* CASE: order-negative-self-physical-exists-count */ FROM post_reads WHERE EXISTS(SELECT 1 FROM post_reads r)) SELECT COUNT(*) FROM post_reads;
WITH early AS (SELECT post_id FROM post_reads), post_reads AS (SELECT DISTINCT post_id /* CASE: order-consumer-forward-physical */ FROM post_reads) SELECT p.id FROM posts p WHERE EXISTS(SELECT * FROM post_reads r WHERE r.post_id=p.id);

WITH watched AS (SELECT DISTINCT post_id /* CASE: order-consumer-nested-shadow */ FROM post_reads) SELECT p.id FROM posts p WHERE EXISTS(SELECT * FROM watched w WHERE w.post_id=p.id) AND EXISTS(WITH watched AS (SELECT NULL::uuid AS post_id) SELECT COUNT(*) FROM watched);

WITH watched AS (SELECT DISTINCT post_id /* CASE: order-negative-consumer-nested-definition */ FROM post_reads) SELECT COUNT(*) FROM (WITH watched AS (SELECT post_id FROM watched) SELECT post_id FROM watched) rebound;

WITH watched AS (SELECT DISTINCT post_id /* CASE: order-consumer-recursive-shadow */ FROM post_reads) SELECT p.id FROM posts p WHERE EXISTS(SELECT * FROM watched w WHERE w.post_id=p.id) AND EXISTS(WITH RECURSIVE watched AS (SELECT NULL::uuid AS post_id UNION ALL SELECT post_id FROM watched WHERE false) SELECT COUNT(*) FROM watched);

WITH watched AS (SELECT DISTINCT post_id /* CASE: order-consumer-recursive-unrelated */ FROM post_reads) SELECT p.id FROM posts p WHERE EXISTS(WITH RECURSIVE walk(n) AS (SELECT 1 UNION ALL SELECT n+1 FROM walk WHERE n<2) SELECT * FROM watched);
