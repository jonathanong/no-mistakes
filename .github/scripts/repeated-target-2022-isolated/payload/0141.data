import sql from "sql-template-strings";
import { read } from "@example/db";

export async function listTopics() {
  const query = sql`SELECT id FROM topics WHERE published = true`;
  query.append(sql` ORDER BY id LIMIT 50`);
  return read(query);
}
