import sql from "sql-template-strings";
import { write } from "@data-stores/psql";

export async function expire(clause: string) {
  const query = sql`WITH stale AS (SELECT id FROM items WHERE expired) `;
  query.append(clause);
  await write(query);
}
