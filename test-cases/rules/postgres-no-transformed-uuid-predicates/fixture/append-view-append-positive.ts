import sql from "sql-template-strings";
import { read } from "db";

export async function load(id: string) {
  const statement = sql`SELECT t.id FROM threads t`;
  statement.append(sql` JOIN view_posts v ON COALESCE(v.id, v.id) = t.id`);
  statement.append(sql` WHERE t.id = ${id}`);
  return read(statement.text, statement.values);
}
