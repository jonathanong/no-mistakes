import sql from "sql-template-strings";
import { write } from "@example/db";

export async function updateTopics(statementBody: string) {
  const query = sql`WITH selected AS (SELECT id FROM topics)`;
  query.append(statementBody);
  return write(query);
}
