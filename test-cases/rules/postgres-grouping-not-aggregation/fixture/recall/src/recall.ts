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
