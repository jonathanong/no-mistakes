import { query, sql } from "@example/db";

function fullFromSql() {
  return sql` FROM accounts a
    JOIN orders o ON o.account_id = a.id`;
}

export function firstFullFrom() {
  return query(
    sql`SELECT a.id`.append(fullFromSql()),
  );
}

export function secondFullFrom() {
  return query(
    sql`SELECT a.id`.append(fullFromSql()),
  );
}

export function suppressedOpening() {
  return query(
    sql`SELECT a.id
      FROM accounts a` // no-mistakes-disable-line postgres-bounded-statements
      .append(remainingSql()),
  );
}

function remainingSql() {
  return sql` JOIN orders o ON o.account_id = a.id
    CROSS JOIN invoices i`;
}

function singleRootFromSql() {
  return sql` FROM invoices`;
}

export function singleRoot() {
  return query(sql`SELECT id`.append(singleRootFromSql()));
}
