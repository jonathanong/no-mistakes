import sql from "sql-template-strings";
import { read } from "db";

export async function load(id: string) {
  const statement = sql`SELECT t.id FROM a t
    JOIN plain_view v ON COALESCE(v.parent_id, v.id) = t.id
    WHERE t.id = ${id} `;
  statement.append(sql` AND t.parent_id IS NOT NULL`);
  statement.append(sql` LIMIT 1`);
  return read(statement);
}
