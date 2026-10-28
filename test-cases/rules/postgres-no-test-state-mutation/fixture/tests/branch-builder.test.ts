import { query, sql } from '@example/db';

export function fixedBranches(flag: boolean, other: boolean) {
  let statement;
  if (flag) statement = sql`CREATE TABLE branch_table (id int)`;
  else if (other) statement = sql`SELECT 1`;
  else throw new Error('no statement');
  return query(statement);
}

export function missingAssignment(flag: boolean) {
  let statement;
  if (flag) statement = sql`CREATE TABLE maybe_table (id int)`;
  return query(statement);
}
