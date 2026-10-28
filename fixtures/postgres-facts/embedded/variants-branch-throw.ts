import { query, sql } from '@example/db';

export function finite(flag: boolean, other: boolean) {
  let statement;
  if (flag) statement = sql`CREATE TABLE finite_branch (id int)`;
  else if (other) statement = sql`SELECT 1`;
  else throw new Error('no query');
  query(statement);
}

export function unassigned(flag: boolean) {
  let statement;
  if (flag) statement = sql`CREATE TABLE maybe_branch (id int)`;
  query(statement);
}

export function escaped(flag: boolean) {
  let statement;
  if (flag) statement = sql`CREATE TABLE escaped_branch (id int)`;
  else throw new Error('no query');
  unknown(statement);
  query(statement);
}

export function shadowed(flag: boolean) {
  let statement;
  if (flag) { let statement = sql`CREATE TABLE inner_branch (id int)`; consume(statement); }
  else throw new Error('no query');
  query(statement);
}

export function caught(flag: boolean) {
  let statement;
  try {
    if (flag) statement = sql`CREATE TABLE caught_branch (id int)`;
    else throw new Error('caught');
  } catch { statement = sql`SELECT 2`; }
  query(statement);
}
