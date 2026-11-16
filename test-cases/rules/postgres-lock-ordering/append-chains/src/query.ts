import { query, sql } from "@example/db";
import sts from "sql-template-strings";

function claimable(now: Date) {
  return sql`w.state = 'ready' AND w.available_at <= ${now}`;
}

function whereHelper(ref: string) {
  return sql`${ref}.archived_at IS NULL`;
}

function helperReturningSql(column: string) {
  return sql`${column}`;
}

function claimableSts(now: Date) {
  return sts`w.state = 'ready' AND w.available_at <= ${now}`;
}

export function claimWork(kind: string, now: Date) {
  return query(
    sql`SELECT w.id FROM work w WHERE w.kind = ${kind} AND `.append(claimable(now)).append(sql` ORDER BY w.id LIMIT 10 FOR UPDATE OF w SKIP LOCKED`),
  );
}

export function lockImage(ref: string, id: string) {
  return query(
    sql`SELECT x.id FROM images x WHERE `.append(whereHelper(ref)).append(sql` AND x.image_id = ${id} ORDER BY x.id FOR UPDATE OF x`),
  );
}

export function claimWorkSts(kind: string, now: Date) {
  return query(
    sts`SELECT w.id FROM work w WHERE w.kind = ${kind} AND `.append(claimableSts(now)).append(sts` ORDER BY w.id LIMIT 10 FOR UPDATE OF w SKIP LOCKED`),
  );
}

export function touchRow(id: string) {
  return query(
    // no-mistakes-disable-next-line postgres-required-predicates
    sql`UPDATE contacts SET seen = true WHERE `.append(helperReturningSql("c.id")).append(sql` = ${id}`),
  );
}
