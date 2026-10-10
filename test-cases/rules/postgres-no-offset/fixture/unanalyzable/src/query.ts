import { query } from "@example/db";

// Opaque executor argument: no SQL text is recovered (unanalyzable, line 5).
export function opaque(sql: string) {
  return query(sql);
}

// Recovered SELECT whose interpolation could append OFFSET (unanalyzable, line 10).
export function sorted(column: string) {
  return query(`SELECT id FROM posts ORDER BY ${column}`);
}

// Dynamic INSERT cannot carry a top-level OFFSET page, so it stays quiet.
export function insertInto(table: string) {
  return query(`INSERT INTO ${table} (id) VALUES ($1)`, ["x"]);
}

// The recovered text already proves OFFSET: one offset finding (line 20), not a second unanalyzable one.
export function paged(offset: number) {
  return query(`SELECT id FROM posts ORDER BY id OFFSET ${offset}`);
}

// Static SQL is fully analyzed and clean.
export function latest() {
  return query(`SELECT id FROM posts ORDER BY id DESC LIMIT 10`);
}

// Line-level suppression applies to unanalyzable findings like any other finding.
export function suppressed(sql: string) {
  // no-mistakes-disable-next-line postgres-no-offset
  return query(sql);
}
