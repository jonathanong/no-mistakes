import { query } from "@example/db";
import sql, { SQLStatement } from "sql-template-strings";

export function loadEntriesWithOpaqueFilter(
  ownerId: string,
  filter: SQLStatement | undefined,
  limit: number,
) {
  const q = sql`SELECT e.* FROM entries e WHERE e.owner_id = ${ownerId}`;
  if (filter) q.append(filter);
  q.append(sql` ORDER BY e.id LIMIT ${limit + 1}`);
  return query(q);
}

export function archiveWithSqlReturning(
  id: string,
  filter: SQLStatement | undefined,
) {
  const q = sql`UPDATE entries SET status = 'archived' WHERE id = ${id}`;
  if (filter) q.append(filter);
  q.append(sql` RETURNING *`);
  return query(q);
}

export function appendPlainStringBaseTable(
  id: string,
  filter: SQLStatement | undefined,
) {
  const q = sql`SELECT id FROM narrow_tags WHERE id = ${id}`;
  if (filter) q.append(filter);
  q.append(" UNION ALL SELECT entries.* FROM entries");
  return query(q);
}

export function appendPlainStringRecords(filter: SQLStatement | undefined) {
  const q = sql`WITH kept AS (SELECT id FROM narrow_tags)`;
  if (filter) q.append(filter);
  q.append(" SELECT records.* FROM records");
  return query(q);
}

export function appendSafeExplicitColumns(
  ownerId: string,
  filter: SQLStatement | undefined,
  limit: number,
) {
  const q = sql`SELECT e.id, e.status FROM entries e WHERE e.owner_id = ${ownerId}`;
  if (filter) q.append(filter);
  q.append(sql` ORDER BY e.id LIMIT ${limit}`);
  return query(q);
}

export function appendWholeRowFunction(
  ownerId: string,
  filter: SQLStatement | undefined,
) {
  const q = sql`SELECT row_to_json(e.*) FROM entries e WHERE e.owner_id = ${ownerId}`;
  if (filter) q.append(filter);
  return query(q);
}
