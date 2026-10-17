import { query, sql } from "@example/db";

export function byColumn(column: string, value: string) {
  return query(sql`SELECT id FROM documents WHERE ${sql.identifier([column])} = ${value}`);
}

export function orderBy(direction: string) {
  return query(sql`SELECT id FROM documents ORDER BY created_at ${sql.raw(direction)}`);
}

export function columns(names: string[]) {
  return query(sql`SELECT ${sql(names)} FROM documents`);
}

export function filters(archived: boolean) {
  return query(sql`SELECT id FROM documents WHERE true ${archived && sql`AND archived_at IS NOT NULL`}`);
}

export function joined() {
  return query(sql`SELECT id FROM documents WHERE true ${[sql`AND a`, sql`AND b`]}`);
}

export function appended() {
  return query(sql`SELECT id FROM documents ${sql`WHERE true`.append(sql` AND a`)}`);
}

export function sequenced(audit: () => void) {
  return query(sql`SELECT id FROM documents WHERE true ${(audit(), sql`AND a`)}`);
}

export function spread(parts: string[]) {
  // `parts` is a value; the spread array literal and the hole are not, but
  // the trailing nested fragment still is.
  return query(sql`SELECT id FROM documents WHERE id = ANY(${[...parts, , ...[sql`AND b`]]})`);
}
