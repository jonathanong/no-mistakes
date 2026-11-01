import { query, sql } from "@example/db";

const JOIN_SQL = sql` JOIN orders o ON o.account_id = a.id`;

export function appendedSuppressed() {
  // no-mistakes-disable-next-line postgres-bounded-statements
  return query(
    sql`SELECT a.id FROM accounts a`.append(JOIN_SQL),
  );
}

export function appendedSuppressedOnCallLine() {
  return query( // no-mistakes-disable-line postgres-bounded-statements
    sql`SELECT a.id FROM accounts a`.append(JOIN_SQL),
  );
}
