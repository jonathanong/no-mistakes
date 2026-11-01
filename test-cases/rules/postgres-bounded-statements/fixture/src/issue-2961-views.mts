import { query } from "@example/db";

export function users() {
  return query("SELECT id FROM view_users_public");
}

export function usersCapped() {
  return query("SELECT id FROM view_users_public LIMIT 1");
}

export function missing() {
  return query("SELECT id FROM relation_not_in_catalog");
}

export function collision(id: string) {
  return query("SELECT id FROM active_accounts WHERE id = $1", [id]);
}

export function feed(id: string) {
  return query("SELECT * FROM view_feeds WHERE view_feeds.id = $1", [id]);
}

export function openFeeds() {
  return query("SELECT id FROM view_feeds");
}

export function materialized(owner: string) {
  return query(
    `WITH sel AS MATERIALIZED (
      SELECT id FROM base WHERE owner_id = $1 ORDER BY id DESC LIMIT 1
    )
    SELECT v.* FROM sel JOIN view_base v ON v.id = sel.id`,
    [owner],
  );
}

export function thing(id: string) {
  return query(
    "SELECT t.id FROM things t JOIN view_people p ON p.id = t.updated_by_id WHERE t.id = $1",
    [id],
  );
}

export function unionItems(a: string, b: string) {
  return query(
    `WITH c AS (
      SELECT id FROM items WHERE id = $1
      UNION ALL
      SELECT id FROM items WHERE id = $2
    )
    SELECT v.id FROM c JOIN view_items v ON c.id = v.id`,
    [a, b],
  );
}

export function peopleOnLogs(id: string) {
  return query(
    "SELECT l.id FROM logs l JOIN items i ON true LEFT JOIN view_people v ON v.id = l.person_id WHERE l.id = $1",
    [id],
  );
}

export function counts() {
  return query("SELECT n FROM view_counts");
}

export function statuses() {
  return query("SELECT status FROM view_status");
}
