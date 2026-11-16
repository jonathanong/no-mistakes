import templateSql from "sql-template-strings";
import { randomUUID } from "node:crypto";
import { query, sql } from "@example/db";

const slug = `widget-${randomUUID()}`;
const WIDGET_SLUG = "default-widget";
const SYSTEM_NAME = "system";
const NAME_KEY = "system";
const KEY_PREFIX = "obj-";
const KEY_PREFIX_BUILT = `obj-${randomUUID()}`;
const LABEL = `keep-${randomUUID()}`;

export function insertWidget(name: string) {
  return query(templateSql`INSERT INTO widgets (slug, name) VALUES (${slug}, ${name})`);
}

export function lookupWidget() {
  return query(sql`SELECT id FROM widgets WHERE slug = ${WIDGET_SLUG}`);
}

export function closeReport(ownerId: string) {
  return query(
    templateSql`UPDATE reports SET state = 'closed' WHERE owner_id = ${ownerId} AND tenant_id = $2 AND reviewer_id = (SELECT id FROM users WHERE username = ${SYSTEM_NAME})`,
  );
}

export function closeReportBound() {
  return query(
    sql`UPDATE reports SET state = 'closed' WHERE owner_id = $1 AND tenant_id = $2 AND reviewer_id = (SELECT id FROM users WHERE username = ${SYSTEM_NAME})`,
  );
}

export function matchUser() {
  return query(sql`SELECT u.id FROM users u WHERE LOWER(u.username) = LOWER(${NAME_KEY})`);
}

export function lockObjectLiteral() {
  return query(
    sql`SELECT COALESCE(o.storage_key, ${KEY_PREFIX} || o.id::text) FROM objects o WHERE o.id = $1 ORDER BY o.id FOR UPDATE`,
  );
}

export function lockObjectBuilt(id: string) {
  return query(
    templateSql`SELECT COALESCE(o.storage_key, ${KEY_PREFIX_BUILT} || o.id::text) FROM objects o WHERE o.id = ${id} ORDER BY o.id FOR UPDATE`,
  );
}

export function labelReport(ownerId: string) {
  // no-mistakes-disable-next-line postgres-required-predicates
  return query(templateSql`UPDATE reports SET state = 'closed' WHERE id = ${ownerId} AND label = ${LABEL}`);
}

export function insertWidgetConflict() {
  // no-mistakes-disable-next-line postgres-conflict-ordering
  return query(templateSql`INSERT INTO widgets (id, slug) SELECT input.id, ${LABEL} FROM unnest($1::uuid[]) AS input(id) ON CONFLICT (id) DO NOTHING`);
}
