import { query } from "@data-stores/psql";

export function insert(ids: string[], categories: string[]) {
  return query(`
    INSERT INTO rss_feed_item_categories (rss_feed_item_id, category_text)
    SELECT input.rss_feed_item_id, input.category_text
    FROM unnest($1::uuid[], $2::text[]) AS input(rss_feed_item_id, category_text)
    ORDER BY input.rss_feed_item_id, lower(input.category_text), input.category_text
    ON CONFLICT (rss_feed_item_id, (LOWER(category_text))) DO NOTHING
  `, [ids, categories]);
}
