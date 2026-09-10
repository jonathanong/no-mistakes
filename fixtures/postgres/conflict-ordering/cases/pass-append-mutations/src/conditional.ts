import sql from "sql-template-strings";
import { read } from "@data-stores/psql";

export async function listTopics(flag: boolean) {
  const query = sql`SELECT id FROM topics WHERE published = true`;
  // Recovered SELECT stays Dynamic so conflict-ordering ignores it.
  if (flag) {
    query.append(sql` ORDER BY id`);
  }
  return read(query);
}
