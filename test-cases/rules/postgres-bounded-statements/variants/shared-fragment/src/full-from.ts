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

export function secondSingleRoot() {
  return query(sql`SELECT id`.append(singleRootFromSql()));
}

export function suppressedSingleRoot() {
  // no-mistakes-disable-next-line postgres-bounded-statements
  return query(sql`SELECT id`.append(singleRootFromSql()));
}

function singleJoinSql() {
  return sql` CROSS JOIN invoices i`;
}

export function boundedOpening(id: string) {
  return query(sql`SELECT i.id FROM accounts a`.append(singleJoinSql()).append(sql` WHERE a.id = ${id}`));
}

export function directSingleRoot() {
  return query(sql`SELECT id FROM invoices`);
}

export function escapedOpening() {
  const statement = sql`SELECT id\nFROM invoices`;
  return query(statement.append(sql` ORDER BY id`));
}

export function firstLaterFragment() {
  return query(sql`SELECT id`.append(laterFromSql()));
}

export function secondLaterFragment() {
  return query(sql`SELECT id`.append(laterFromSql()));
}

function laterFromSql() {
  return sql` FROM invoices`;
}

export function emptyOpening() {
  return query(sql``.append(laterStatementSql()));
}

export function secondEmptyOpening() {
  return query(sql``.append(laterStatementSql()));
}

function laterStatementSql() {
  return sql`SELECT id FROM invoices`;
}

export function physicalNewlineOpening() {
  const statement = sql`SELECT id
    FROM invoices`;
  return query(statement.append(sql` ORDER BY id`));
}
