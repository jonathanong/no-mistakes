import { query, sql } from "db";

export async function sameLine() {
  const unicode = "é"; await query(sql`SELECT DISTINCT id /* CASE: embedded-same-line-first */ FROM posts`); await query(sql`SELECT DISTINCT id /* CASE: embedded-same-line-second */ FROM posts`);
  void unicode;
  const unicodeAppend = "é"; const first = sql`SELECT DISTINCT id /* CASE: append-same-line-first */ `; first.append(sql`FROM posts`); await query(first); const second = sql`SELECT DISTINCT id /* CASE: append-same-line-second */ `; second.append(sql`FROM posts`); await query(second); void unicodeAppend;
}
