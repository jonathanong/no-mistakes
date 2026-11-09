import {query, sql} from "db";
query(sql`SELECT DISTINCT author_id,parent_id /* CASE: tail-negative-nullable-multi-rollup */ FROM posts GROUP BY ROLLUP(author_id,parent_id)`);

query(sql`SELECT DISTINCT n.slug /* CASE: tail-negative-nullable-joined-rollup */ FROM nullable_equal_slugs n INNER JOIN post_reads r ON true GROUP BY ROLLUP(n.slug)`);

query(sql`SELECT DISTINCT slug /* CASE: tail-reverse-equality-rollup */ FROM nullable_equal_slugs WHERE $1::text=slug GROUP BY ROLLUP(slug)`);

query(sql`SELECT DISTINCT slug /* CASE: tail-negative-null-input-rollup */ FROM nullable_equal_slugs WHERE slug IS NULL GROUP BY ROLLUP(slug)`);

query(sql`WITH watched AS (SELECT DISTINCT post_id /* CASE: tail-consumer-unrelated-no-from */ FROM post_reads) SELECT p.id FROM posts p WHERE EXISTS(SELECT * FROM watched w WHERE w.post_id=p.id) AND EXISTS(SELECT 1)`);

query(sql`WITH watched AS (SELECT DISTINCT post_id /* CASE: tail-consumer-unrelated-values */ FROM post_reads) SELECT p.id FROM posts p WHERE EXISTS(SELECT * FROM watched w WHERE w.post_id=p.id) AND EXISTS(SELECT 1 FROM (VALUES(1))v(n))`);

query(sql`WITH watched AS (SELECT DISTINCT post_id /* CASE: tail-consumer-unrelated-union */ FROM post_reads) SELECT p.id FROM posts p WHERE EXISTS(SELECT * FROM watched w WHERE w.post_id=p.id) AND EXISTS(SELECT 1 UNION SELECT 2)`);

query(sql`SELECT 1>0 /* CASE: tail-negative-no-count */ FROM post_reads`);

export async function nullableCount(value: string | null) {
  await query(sql`SELECT COUNT(${value})>0 /* CASE: tail-negative-nullable-count-bind */ FROM post_reads`);
}
