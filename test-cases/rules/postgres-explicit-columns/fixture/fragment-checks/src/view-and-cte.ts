import { query } from "@example/db";
import sql, { SQLStatement } from "sql-template-strings";

export function selectFromViewBuilder(
  ownerId: string,
  filter: SQLStatement | undefined,
) {
  const q = sql`SELECT v.* FROM entries_summary v WHERE v.owner_id = ${ownerId}`;
  if (filter) q.append(filter);
  return query(q);
}

export function appendPlainStringView(filter: SQLStatement | undefined) {
  const q = sql`SELECT id FROM narrow_tags`;
  if (filter) q.append(filter);
  q.append(" UNION ALL SELECT v.* FROM audit_view v");
  return query(q);
}

export function selectFromCteBuilder(filter: SQLStatement | undefined) {
  const q = sql`WITH recent AS (SELECT id FROM entries) SELECT recent.* FROM recent`;
  if (filter) q.append(filter);
  return query(q);
}

export function appendPlainStringCteFromPrefix(
  id: string,
  filter: SQLStatement | undefined,
) {
  const q = sql`WITH upserted AS (INSERT INTO entries (id) VALUES (${id}) RETURNING id)`;
  if (filter) q.append(filter);
  q.append(" SELECT upserted.* FROM upserted");
  return query(q);
}
