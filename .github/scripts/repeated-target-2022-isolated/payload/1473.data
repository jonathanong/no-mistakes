import sql from "sql-template-strings";
import { read } from "@example/db";

export async function listTopics(clause: string) {
  const query = sql`SELECT id FROM topics WHERE published = true`;
  query.append(clause);
  return read(query);
}
