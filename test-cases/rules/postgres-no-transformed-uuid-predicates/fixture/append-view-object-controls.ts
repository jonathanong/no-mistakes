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
