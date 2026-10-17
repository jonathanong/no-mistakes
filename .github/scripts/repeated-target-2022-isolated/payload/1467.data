import sql from "sql-template-strings";
import { write } from "@example/db";

export async function updateTopic(id: string, title: string) {
  const query = sql`UPDATE topics SET title = ${title} WHERE id = ${id}`;
  query.append(sql` AND published = true`);
  return write(query);
}
