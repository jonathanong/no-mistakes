import sql from "sql-template-strings";
import { read } from "db";

export function aliasResult() {
  const initial = sql`SELECT id FROM threads`;
  const alias = initial.append(sql` LIMIT 1`);
  return read(alias);
}

export function untouched() {
  const statement = sql`SELECT id FROM threads`;
  return read(statement);
}

export function conditional(flag: boolean) {
  const statement = sql`SELECT id FROM threads`;
  if (flag) statement.append(sql` WHERE published = true`);
  return read(statement);
}
