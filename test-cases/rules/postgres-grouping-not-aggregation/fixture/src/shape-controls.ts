import { query, sql } from "db";

export async function allShapes(id: string, slug: string, tags: string[]) {
  await query(sql`
    SELECT candidate.item_id /* CASE: embedded-all-of-distinct */
    FROM (
      SELECT item_id, tag_id FROM direct_tags WHERE tag_id = ANY(${tags}::uuid[])
      UNION
      SELECT item_id, tag_id FROM alias_tags WHERE tag_id = ANY(${tags}::uuid[])
    ) AS candidate
    GROUP BY candidate.item_id
    HAVING COUNT(DISTINCT candidate.tag_id) = cardinality(${tags}::uuid[])
  `);
  await query(sql`SELECT DISTINCT post_id /* CASE: embedded-unique-composite-distinct */ FROM topic_memberships WHERE topic_id = ${id}`);
  await query(sql`SELECT post_id /* CASE: embedded-unique-composite-group */ FROM topic_memberships WHERE topic_id = ${id} GROUP BY post_id`);
  await query(sql`SELECT DISTINCT id /* CASE: embedded-unique-primary-key */ FROM posts`);
  await query(sql`SELECT COUNT(DISTINCT id) /* CASE: embedded-unique-count-distinct */ FROM posts`);
  await query(sql`SELECT DISTINCT topic_id /* CASE: embedded-unique-partial */ FROM live_topics WHERE deleted_at IS NULL`);
  await query(sql`SELECT DISTINCT topic_id /* CASE: embedded-negative-partial-or */ FROM live_topics WHERE deleted_at IS NULL OR topic_id = ${id}`);
  await query(sql`SELECT DISTINCT topic_id FROM live_topics`);
  await query(sql`SELECT DISTINCT slug FROM nullable_slugs WHERE slug IS NULL`);
  await query(sql`SELECT DISTINCT slug FROM nullable_slugs`);
  await query(sql`
    SELECT DISTINCT ON (r.input_order) p.id /* CASE: embedded-unique-distinct-on-join */
    FROM request_items AS r JOIN posts AS p ON p.slug = r.slug
    ORDER BY r.input_order
  `);
  await query(sql`
    SELECT DISTINCT id /* CASE: embedded-union-derived */ FROM (
      SELECT id FROM current_posts UNION SELECT id FROM archived_posts
    ) AS both_posts
  `);
  await query(sql`
    WITH both_posts AS (
      SELECT DISTINCT id /* CASE: embedded-union-arm-left */ FROM current_posts
      UNION SELECT DISTINCT id /* CASE: embedded-union-arm-right */ FROM archived_posts
    ) SELECT id FROM both_posts
  `);
  await query(sql`
    WITH both_posts AS (
      SELECT id FROM current_posts UNION SELECT id FROM archived_posts
    ) SELECT DISTINCT id /* CASE: embedded-union-cte */ FROM both_posts
  `);
  await query(sql`
    SELECT p.id /* CASE: embedded-filter-derived-join */ FROM posts AS p
    JOIN (SELECT DISTINCT post_id FROM post_reads WHERE reader_id = ${id}) AS reads
      ON reads.post_id = p.id
  `);
  await query(sql`
    SELECT p.id /* CASE: embedded-negative-partial-join-key */ FROM posts AS p
    JOIN (SELECT DISTINCT post_id, reader_id FROM post_reads) AS reads ON reads.post_id = p.id
  `);
  await query(sql`
    SELECT p.id /* CASE: embedded-negative-derived-later-join */ FROM posts AS p
    JOIN (SELECT DISTINCT post_id, reader_id FROM post_reads) AS reads ON reads.post_id = p.id
    JOIN posts AS reader ON reader.author_id = reads.reader_id
  `);
  await query(sql`
    SELECT p.id /* CASE: embedded-negative-derived-order-by */ FROM posts AS p
    JOIN (SELECT DISTINCT post_id, created_at FROM post_reads) AS reads ON reads.post_id = p.id
    ORDER BY reads.created_at
  `);
  await query(sql`
    SELECT p.id /* CASE: embedded-filter-all-membership */ FROM posts AS p
    WHERE p.id <> ALL(ARRAY(SELECT DISTINCT post_id FROM blocked_posts WHERE active))
  `);
  await query(sql`
    WITH watched AS (SELECT DISTINCT post_id FROM post_reads WHERE reader_id = ${id})
    SELECT p.id /* CASE: embedded-filter-cte-in */ FROM posts AS p WHERE p.id IN (SELECT post_id FROM watched)
  `);
  await query(sql`
    WITH watched AS (SELECT DISTINCT post_id FROM post_reads WHERE reader_id = ${id})
    SELECT p.id /* CASE: embedded-filter-cte-not-exists */ FROM posts AS p
    WHERE NOT EXISTS (SELECT 1 FROM watched AS w WHERE w.post_id = p.id)
  `);
  await query(sql`
    SELECT p.author_id, COUNT(DISTINCT p.id) AS post_count /* CASE: embedded-filter-count-distinct */
    FROM posts AS p JOIN post_tags AS t ON t.post_id = p.id
    WHERE t.tag_id = ANY(${tags}::uuid[]) GROUP BY p.author_id
  `);
  await query(sql`
    SELECT p.id /* CASE: embedded-count-positive */ FROM posts AS p
    WHERE (SELECT COUNT(*) FROM post_reads AS r WHERE r.post_id = p.id) > 0
  `);
  await query(sql`
    SELECT p.id /* CASE: embedded-count-zero */ FROM posts AS p
    WHERE (SELECT COUNT(*) FROM post_reads AS r WHERE r.post_id = p.id) = 0
  `);
  await query(sql`SELECT COUNT(*) > 0 /* CASE: embedded-count-direct-positive */ FROM post_reads WHERE reader_id = ${id}`);
  await query(sql`SELECT COUNT(*) = 0 /* CASE: embedded-count-direct-zero */ FROM post_reads WHERE reader_id = ${id}`);
  await query(sql`
    SELECT r.post_id /* CASE: embedded-aggregate-not-exists-max */ FROM post_reads AS r GROUP BY r.post_id
    HAVING NOT EXISTS (
      SELECT 1 FROM post_reads AS newer
      WHERE newer.post_id = r.post_id AND newer.created_at >= ${id}
    ) OR MAX(r.created_at) < ${id}
  `);
  await query(sql`SELECT topic_id, COUNT(*) AS rating_count, SUM(score) AS total_score FROM ratings GROUP BY topic_id`);
  await query(sql`SELECT author_id, COUNT(*) AS post_count, MAX(created_at) AS latest_post FROM posts GROUP BY author_id`);
  await query(sql`
    WITH requested_posts AS (
      SELECT id, 'id'::text AS source FROM posts WHERE id = ${id}
      UNION SELECT id, 'slug'::text AS source FROM posts WHERE slug = ${slug}
    ) SELECT DISTINCT id /* CASE: embedded-negative-requested-union */ FROM requested_posts
  `);
  await query(sql`
    SELECT DISTINCT id /* CASE: embedded-negative-union-all-distinct */ FROM (
      SELECT id FROM current_posts UNION ALL SELECT id FROM archived_posts
    ) AS all_posts
  `);
  await query(sql`
    SELECT id, 'current'::text AS source /* CASE: embedded-negative-union-all-projection */ FROM current_posts
    UNION ALL SELECT id, 'archived'::text AS source FROM archived_posts
  `);
  await query(sql`
    SELECT DISTINCT p.id, reads.created_at /* CASE: embedded-negative-projected-join-side */
    FROM posts AS p
    JOIN (SELECT DISTINCT post_id, created_at FROM post_reads) AS reads ON reads.post_id = p.id
  `);
  await query(sql`
    SELECT p.id /* CASE: embedded-negative-limited-membership */ FROM posts AS p
    WHERE p.id IN (
      SELECT DISTINCT post_id FROM post_reads ORDER BY post_id LIMIT 10 OFFSET 2
    )
  `);
  await query(sql`
    WITH watched AS (SELECT DISTINCT post_id FROM post_reads)
    SELECT p.id FROM posts AS p WHERE p.id IN (SELECT post_id FROM watched)
    UNION ALL
    SELECT watched.post_id FROM watched JOIN posts AS p ON p.id = watched.post_id
  `);
  await query(sql`
    SELECT DISTINCT p.id /* CASE: embedded-negative-outer-join-unique-driver */
    FROM posts AS p LEFT JOIN post_reads AS reads ON reads.post_id = p.id
  `);
  await query(sql`SELECT DISTINCT ON (author_id) id, author_id, created_at FROM posts ORDER BY author_id, created_at DESC`);
  await query(sql`
    WITH RECURSIVE ancestors(id, parent_id) AS (
      SELECT id, parent_id FROM posts WHERE id = ${id}
      UNION ALL
      SELECT p.id, p.parent_id FROM posts AS p JOIN ancestors AS a ON p.id = a.parent_id
    ) SELECT DISTINCT id /* CASE: embedded-negative-recursive */ FROM ancestors
  `);
  await query(sql`
    SELECT p.author_id, COUNT(DISTINCT p.id), MAX(t.created_at)
    FROM posts AS p JOIN post_tags AS t ON t.post_id = p.id GROUP BY p.author_id
  `);
  await query(sql`
    SELECT author_id,
      COUNT(*) FILTER (WHERE created_at >= ${id}) OVER (PARTITION BY author_id) AS recent_count,
      COUNT(*) OVER (PARTITION BY author_id) AS all_count
    FROM posts
  `);
}

export async function appendedCandidatePool(tags: string[]) {
  const statement = sql`
    SELECT candidate.item_id /* CASE: embedded-append-all-of-distinct */ FROM (
      SELECT item_id, tag_id FROM direct_tags WHERE tag_id = ANY(${tags}::uuid[])
      UNION SELECT item_id, tag_id FROM alias_tags WHERE tag_id = ANY(${tags}::uuid[])
    ) AS candidate
  `;
  statement.append(sql`
    GROUP BY candidate.item_id
    HAVING COUNT(DISTINCT candidate.tag_id) = cardinality(${tags}::uuid[])
  `);
  await query(statement);
}

export async function appendedUniqueKey(topicId: string) {
  const statement = sql`SELECT DISTINCT topic_id /* CASE: embedded-append-unique-partial */ FROM live_topics`;
  statement.append(sql` WHERE deleted_at IS NULL`);
  await query(statement);
}

export async function appendedUnion() {
  const statement = sql`SELECT DISTINCT id /* CASE: embedded-append-union */ FROM (SELECT id FROM current_posts `;
  statement.append(sql`UNION SELECT id FROM archived_posts) AS both_posts`);
  await query(statement);
}

export async function appendedJoinFilter(readerId: string) {
  const statement = sql`SELECT p.id /* CASE: embedded-append-filter-join */ FROM posts AS p JOIN (`;
  statement.append(sql`SELECT DISTINCT post_id FROM post_reads WHERE reader_id = ${readerId}`);
  statement.append(sql`) AS reads ON reads.post_id = p.id`);
  await query(statement);
}

export async function appendedAggregateExistence() {
  const statement = sql`SELECT p.id /* CASE: embedded-append-count-positive */ FROM posts AS p `;
  statement.append(sql`WHERE (SELECT COUNT(*) FROM post_reads AS r WHERE r.post_id = p.id) > 0`);
  await query(statement);
}

export async function appendedDistinctCount() {
  const statement = sql`SELECT COUNT(DISTINCT id) /* CASE: embedded-append-unique-count-distinct */ `;
  statement.append(sql`FROM posts`);
  await query(statement);
}

export async function appendedAllOfCountStar(tags: string[]) {
  const statement = sql`SELECT item_id /* CASE: embedded-append-all-of-count-star */ FROM direct_tags `;
  statement.append(sql`WHERE tag_id = ANY(${tags}::uuid[]) GROUP BY item_id HAVING COUNT(*) = ${tags.length}`);
  await query(statement);
}

export async function appendedCompositeGroup(topicId: string) {
  const statement = sql`SELECT post_id /* CASE: embedded-append-unique-composite-group */ FROM topic_memberships `;
  statement.append(sql`WHERE topic_id = ${topicId} GROUP BY post_id`);
  await query(statement);
}

export async function appendedDistinctOnJoin() {
  const statement = sql`SELECT DISTINCT ON (r.input_order) p.id /* CASE: embedded-append-unique-distinct-on-join */ FROM request_items AS r `;
  statement.append(sql`JOIN posts AS p ON p.slug = r.slug ORDER BY r.input_order`);
  await query(statement);
}

export async function appendedUnionCte() {
  const statement = sql`WITH both_posts AS (SELECT id FROM current_posts `;
  statement.append(sql`UNION SELECT id FROM archived_posts)`);
  statement.append(sql` SELECT DISTINCT id /* CASE: embedded-append-union-cte */ FROM both_posts`);
  await query(statement);
}

export async function appendedUnionArms() {
  const statement = sql`WITH both_posts AS (SELECT DISTINCT id /* CASE: embedded-append-union-arm-left */ FROM current_posts `;
  statement.append(sql`UNION SELECT DISTINCT id /* CASE: embedded-append-union-arm-right */ FROM archived_posts)`);
  statement.append(sql` SELECT id FROM both_posts`);
  await query(statement);
}

export async function appendedCteNotExists(readerId: string) {
  const statement = sql`WITH watched AS (SELECT DISTINCT post_id FROM post_reads `;
  statement.append(sql`WHERE reader_id = ${readerId})`);
  statement.append(sql` SELECT p.id /* CASE: embedded-append-filter-cte-not-exists */ FROM posts AS p WHERE NOT EXISTS (SELECT 1 FROM watched w WHERE w.post_id = p.id)`);
  await query(statement);
}

export async function appendedCountZero() {
  const statement = sql`SELECT p.id /* CASE: embedded-append-count-zero */ FROM posts AS p `;
  statement.append(sql`WHERE (SELECT COUNT(*) FROM post_reads AS r WHERE r.post_id = p.id) = 0`);
  await query(statement);
}

export async function appendedDirectCounts(readerId: string) {
  const positive = sql`SELECT COUNT(*) > 0 /* CASE: embedded-append-count-direct-positive */ `;
  positive.append(sql`FROM post_reads WHERE reader_id = ${readerId}`);
  const zero = sql`SELECT COUNT(*) = 0 /* CASE: embedded-append-count-direct-zero */ `;
  zero.append(sql`FROM post_reads WHERE reader_id = ${readerId}`);
  await query(positive);
  await query(zero);
}

export async function appendedLimitedMembership() {
  const statement = sql`SELECT p.id /* CASE: embedded-append-negative-limited-membership */ FROM posts AS p WHERE p.id IN (`;
  statement.append(sql`SELECT DISTINCT post_id FROM post_reads ORDER BY post_id LIMIT 10 OFFSET 2)`);
  await query(statement);
}
