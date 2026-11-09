import {query, sql} from "db";
query(sql`SELECT /* CASE: scope-negative-candidate-values */ item_id FROM (
 (WITH direct_tags(item_id,tag_id) AS (VALUES(NULL::uuid,NULL::uuid)) SELECT item_id,tag_id FROM direct_tags)
 UNION (WITH alias_tags(item_id,tag_id) AS (VALUES(NULL::uuid,NULL::uuid)) SELECT item_id,tag_id FROM alias_tags)
) pool GROUP BY item_id HAVING COUNT(*)=$1`);

query(sql`(WITH posts(id) AS (VALUES(NULL::uuid)) SELECT /* CASE: scope-negative-union-arm-values */ DISTINCT id FROM posts) UNION SELECT NULL::uuid`);

query(sql`SELECT /* CASE: scope-negative-union-outer-values */ DISTINCT id FROM (WITH current_posts(id) AS (VALUES(NULL::uuid)) SELECT id FROM current_posts UNION SELECT NULL::uuid) pool`);

query(sql`SELECT p.id FROM posts p WHERE EXISTS (WITH post_reads(post_id) AS (VALUES(NULL::uuid)) SELECT /* CASE: scope-negative-membership-values */ DISTINCT post_id FROM post_reads)`);

query(sql`WITH watched AS (WITH post_reads(post_id) AS (VALUES(NULL::uuid)) SELECT /* CASE: scope-negative-consumer-values */ DISTINCT post_id FROM post_reads) SELECT p.id FROM posts p WHERE EXISTS (SELECT * FROM watched w WHERE w.post_id=p.id)`);

query(sql`SELECT (WITH post_reads AS (SELECT NULL::uuid AS post_id) SELECT /* CASE: scope-negative-scalar-nullable */ COUNT(post_id) FROM post_reads)>0`);

query(sql`SELECT /* CASE: scope-candidate-unrelated-with */ item_id FROM (WITH ignored AS (VALUES(1)) SELECT item_id,tag_id FROM direct_tags UNION SELECT item_id,tag_id FROM alias_tags) pool GROUP BY item_id HAVING COUNT(*)=$1`);

query(sql`SELECT /* CASE: scope-candidate-qualified-physical */ item_id FROM ((WITH direct_tags AS (VALUES(1)) SELECT item_id,tag_id FROM public.direct_tags) UNION (WITH alias_tags AS (VALUES(1)) SELECT item_id,tag_id FROM public.alias_tags)) pool GROUP BY item_id HAVING COUNT(*)=$1`);

query(sql`(WITH ignored AS (VALUES(1)) SELECT /* CASE: scope-union-arm-unrelated-with */ DISTINCT post_id FROM post_reads) UNION SELECT NULL::uuid`);

query(sql`SELECT /* CASE: scope-union-outer-qualified-physical */ DISTINCT id FROM (WITH current_posts(id) AS (VALUES(NULL::uuid)) SELECT id FROM public.current_posts UNION SELECT NULL::uuid) pool`);

query(sql`SELECT p.id FROM posts p WHERE EXISTS (WITH ignored AS (VALUES(1)) SELECT /* CASE: scope-membership-unrelated-with */ DISTINCT post_id FROM post_reads)`);

query(sql`SELECT (WITH ignored AS (VALUES(1)) SELECT /* CASE: scope-scalar-unrelated-with */ COUNT(post_id) FROM post_reads)>0`);
