import sql from "sql-template-strings";
import { read } from "@data-stores/psql";

export async function listTopics(clause: string) {
  const query = sql`SELECT id FROM topics WHERE published = true`;
  query.append(clause);
  return read(query);
}
