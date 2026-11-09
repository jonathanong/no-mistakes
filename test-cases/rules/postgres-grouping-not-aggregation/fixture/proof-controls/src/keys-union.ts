import { query, sql } from "db";

export async function embeddedCoverage(tags: string[], id: string) {
  await query(sql`
    SELECT item_id /* CASE: coverage-embedded-any-reverse-threshold */
    FROM direct_tags WHERE tag_id = ANY(${tags}::uuid[])
    GROUP BY item_id HAVING cardinality(${tags}::uuid[]) = COUNT(*)
  `);
  await query(sql`
    SELECT item_id /* CASE: coverage-embedded-any-distinct-tag */
    FROM direct_tags WHERE tag_id = ANY(${tags}::uuid[])
    GROUP BY item_id HAVING COUNT(DISTINCT tag_id) = cardinality(${tags}::uuid[])
  `);
  await query(sql`
    SELECT DISTINCT unioned.subject_id /* CASE: coverage-embedded-union-alias */
    FROM (
      SELECT topic_id AS subject_id FROM ratings
      UNION SELECT topic_id AS subject_id FROM ratings
    ) AS unioned
  `);
  await query(sql`
    SELECT DISTINCT unioned.* /* CASE: coverage-embedded-union-wildcard */
    FROM (
      SELECT topic_id, score FROM ratings
      UNION SELECT topic_id, score FROM ratings
    ) AS unioned
  `);
  await query(sql`
    SELECT DISTINCT slug /* CASE: coverage-embedded-unique-is-not-null */
    FROM nullable_slugs WHERE slug IS NOT NULL
  `);
  await query(sql`
    SELECT post.id
    FROM posts AS post
    JOIN (SELECT DISTINCT post_id AS target_id /* CASE: coverage-derived-output-alias */ FROM post_reads) AS reads
      ON reads.target_id = post.id
  `);
  await query(sql`
    SELECT DISTINCT live.topic_id /* CASE: coverage-embedded-partial-index-foreign */
    FROM live_topics AS live JOIN posts AS post ON post.id = live.topic_id
    WHERE post.id = ${id}::uuid AND ${id} IS NOT NULL AND live.deleted_at IS NULL
  `);
  await query(sql`
    SELECT d.item_id /* CASE: coverage-negative-embedded-unrelated-count */
    FROM direct_tags AS d WHERE d.tag_id = ANY(${tags}::uuid[])
    GROUP BY d.item_id HAVING COUNT(DISTINCT d.item_id) = cardinality(${tags}::uuid[])
  `);
  await query(sql`
    SELECT post.id /* CASE: coverage-negative-embedded-join-fanout */
    FROM posts AS post JOIN post_reads AS reads ON reads.post_id = post.id
  `);
}

export async function appendCoverage(tags: string[]) {
  const candidate = sql`SELECT item_id /* CASE: coverage-append-any-distinct-tag */ FROM direct_tags `;
  candidate.append(sql`WHERE tag_id = ANY(${tags}::uuid[]) GROUP BY item_id `);
  candidate.append(sql`HAVING COUNT(DISTINCT tag_id) = cardinality(${tags}::uuid[])`);
  await query(candidate);

  const union = sql`SELECT DISTINCT unioned.* /* CASE: coverage-append-union-wildcard */ FROM (`;
  union.append(sql`SELECT topic_id, score FROM ratings UNION SELECT topic_id, score FROM ratings`);
  union.append(sql`) AS unioned`);
  await query(union);

  const derived = sql`SELECT post.id FROM posts AS post JOIN (`;
  derived.append(sql`SELECT DISTINCT post_id AS target_id /* CASE: coverage-append-derived-output-alias */ FROM post_reads`);
  derived.append(sql`) AS reads ON reads.target_id = post.id`);
  await query(derived);

  const negative = sql`SELECT d.item_id /* CASE: coverage-negative-append-unrelated-count */ FROM direct_tags AS d `;
  negative.append(sql`WHERE d.tag_id = ANY(${tags}::uuid[]) GROUP BY d.item_id `);
  negative.append(sql`HAVING COUNT(DISTINCT d.item_id) = cardinality(${tags}::uuid[])`);
  await query(negative);
}

export async function orderedCountCoverage() {
  await query(sql`SELECT COUNT(DISTINCT id ORDER BY id) /* CASE: coverage-embedded-ordered-count-distinct */ FROM posts`);
}
