import { query, sql } from "db";

export async function recall() {
  await query(sql`SELECT COUNT(DISTINCT slug) /* CASE: embedded-recall-nullable-count-distinct */ FROM nullable_slugs`);
  await query(sql`
    SELECT DISTINCT * /* CASE: embedded-recall-union-star */ FROM (
      SELECT * FROM current_posts UNION SELECT * FROM archived_posts
    ) AS all_posts
  `);
  await query(sql`
    SELECT DISTINCT id, id::text AS display_id /* CASE: embedded-recall-union-extra-expression */
    FROM (SELECT id FROM current_posts UNION SELECT id FROM archived_posts) AS all_posts
  `);
}

export async function recallAppend() {
  const count = sql`SELECT COUNT(DISTINCT slug) /* CASE: append-recall-nullable-count-distinct */ `;
  count.append(sql`FROM nullable_slugs`);
  const star = sql`SELECT DISTINCT * /* CASE: append-recall-union-star */ FROM (`;
  star.append(sql`SELECT * FROM current_posts UNION SELECT * FROM archived_posts) AS all_posts`);
  const expression = sql`SELECT DISTINCT id, id::text AS display_id /* CASE: append-recall-union-extra-expression */ FROM (`;
  expression.append(sql`SELECT id FROM current_posts UNION SELECT id FROM archived_posts) AS all_posts`);
  await query(count);
  await query(star);
  await query(expression);
}

export async function recallAggregates() {
  await query(sql`
    SELECT topic_id, COUNT(DISTINCT post_id) /* CASE: embedded-recall-composite-count-distinct */
    FROM topic_memberships GROUP BY topic_id
  `);
  await query(sql`
    SELECT COUNT(DISTINCT r.input_order) /* CASE: embedded-recall-one-to-one-join-count */
    FROM request_items AS r JOIN posts AS p ON p.slug = r.slug
  `);
  await query(sql`
    SELECT m.topic_id, COUNT(DISTINCT m.post_id) /* CASE: embedded-recall-fanout-count-distinct */
    FROM topic_memberships AS m JOIN post_tags AS t ON t.post_id = m.post_id
    GROUP BY m.topic_id
  `);
  await query(sql`
    SELECT id /* CASE: embedded-recall-having-distinct-id */ FROM posts
    GROUP BY id HAVING COUNT(DISTINCT id) = 1
  `);
  await query(sql`SELECT COUNT(DISTINCT id) FILTER (WHERE author_id IS NOT NULL) /* CASE: embedded-recall-filtered-distinct-id */ FROM posts`);
  await query(sql`
    SELECT p.id /* CASE: embedded-recall-negative-scalar-author-count */,
      (SELECT COUNT(DISTINCT author_id) FROM posts) AS author_count
    FROM posts AS p
  `);
  await query(sql`SELECT (SELECT COUNT(DISTINCT author_id) /* CASE: embedded-recall-count-distinct-existence */ FROM posts) > 0`);
  await query(sql`SELECT slug /* CASE: embedded-recall-grouped-key-projection */ FROM posts GROUP BY id`);
}

export async function recallAggregateAppend() {
  const composite = sql`SELECT topic_id, COUNT(DISTINCT post_id) /* CASE: append-recall-composite-count-distinct */ `;
  composite.append(sql`FROM topic_memberships GROUP BY topic_id`);
  const joined = sql`SELECT COUNT(DISTINCT r.input_order) /* CASE: append-recall-one-to-one-join-count */ `;
  joined.append(sql`FROM request_items AS r JOIN posts AS p ON p.slug = r.slug`);
  const fanout = sql`SELECT m.topic_id, COUNT(DISTINCT m.post_id) /* CASE: append-recall-fanout-count-distinct */ `;
  fanout.append(sql`FROM topic_memberships AS m JOIN post_tags AS t ON t.post_id = m.post_id GROUP BY m.topic_id`);
  const having = sql`SELECT id /* CASE: append-recall-having-distinct-id */ FROM posts `;
  having.append(sql`GROUP BY id HAVING COUNT(DISTINCT id) = 1`);
  const filtered = sql`SELECT COUNT(DISTINCT id) FILTER (WHERE author_id IS NOT NULL) /* CASE: append-recall-filtered-distinct-id */ `;
  filtered.append(sql`FROM posts`);
  const negative = sql`SELECT p.id /* CASE: append-recall-negative-scalar-author-count */, `;
  negative.append(sql`(SELECT COUNT(DISTINCT author_id) FROM posts) AS author_count FROM posts AS p`);
  const existence = sql`SELECT (SELECT COUNT(DISTINCT author_id) /* CASE: append-recall-count-distinct-existence */ `;
  existence.append(sql`FROM posts) > 0`);
  const grouped = sql`SELECT slug /* CASE: append-recall-grouped-key-projection */ FROM posts `;
  grouped.append(sql`GROUP BY id`);
  await query(composite);
  await query(joined);
  await query(fanout);
  await query(having);
  await query(filtered);
  await query(negative);
  await query(existence);
  await query(grouped);
}

export async function recursiveUnrelatedKey(id: string) {
  await query(sql`
    WITH RECURSIVE walk AS (
      SELECT id, parent_id FROM posts WHERE id = ${id}
      UNION ALL
      SELECT p.id, p.parent_id FROM posts AS p JOIN walk AS w ON p.id = w.parent_id
    )
    SELECT DISTINCT id /* CASE: embedded-recall-recursive-unrelated-key */ FROM posts
  `);
}

export async function recursiveUnrelatedKeyAppend(id: string) {
  const statement = sql`WITH RECURSIVE walk AS (
    SELECT id, parent_id FROM posts WHERE id = ${id}
  `;
  statement.append(sql`UNION ALL SELECT p.id, p.parent_id FROM posts AS p JOIN walk AS w ON p.id = w.parent_id)`);
  statement.append(sql` SELECT DISTINCT id /* CASE: append-recall-recursive-unrelated-key */ FROM posts`);
  await query(statement);
}

export async function directAnyDistinctPool(tags: string[]) {
  await query(sql`
    SELECT item_id /* CASE: embedded-recall-any-distinct-tag-pool */
    FROM direct_tags
    WHERE tag_id = ANY(${tags}::uuid[])
    GROUP BY item_id
    HAVING COUNT(DISTINCT tag_id) = cardinality(${tags}::uuid[])
  `);
}

export async function directAnyDistinctPoolAppend(tags: string[]) {
  const statement = sql`SELECT item_id /* CASE: append-recall-any-distinct-tag-pool */ FROM direct_tags `;
  statement.append(sql`WHERE tag_id = ANY(${tags}::uuid[]) GROUP BY item_id `);
  statement.append(sql`HAVING COUNT(DISTINCT tag_id) = cardinality(${tags}::uuid[])`);
  await query(statement);
}
