import sql from "sql-template-strings";
import { read } from "db";

export async function load(id: string) {
  return read(`SELECT t.id FROM threads t JOIN view_posts v ON v.id = COALESCE(t.root_id, t.id) WHERE t.id = $1`, [id]);
}
