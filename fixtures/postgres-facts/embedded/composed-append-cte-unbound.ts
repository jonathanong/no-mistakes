import sql from "sql-template-strings";
import { write } from "@data-stores/psql";

export async function updateTopics(statementBody: string) {
  const query = sql`WITH selected AS (SELECT id FROM topics)`;
  query.append(statementBody);
  return write(query);
}
