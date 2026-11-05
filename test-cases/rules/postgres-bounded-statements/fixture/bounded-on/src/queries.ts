import { query, sql } from "./db";

export function items(id: string) {
  return query(sql`SELECT i.id
    FROM items i
    JOIN feeds f ON f.id = i.feed_id AND f.deleted_at IS NULL
    JOIN topics t ON t.id = f.topic_id AND t.deleted_at IS NULL
    WHERE i.id = ${id}`);
}

export function accounts(id: string, status: number) {
  return query(sql`SELECT a.id
    FROM accounts a
    JOIN authorizations z ON z.id = a.authorization_id AND z.status = ${status}
    WHERE a.id = ${id}`);
}

export function participants(id: string) {
  return query(sql`SELECT p.id
    FROM participants p
    JOIN users u ON u.id = p.user_id AND u.deleted_at IS NULL
    WHERE p.id = ${id}`);
}
