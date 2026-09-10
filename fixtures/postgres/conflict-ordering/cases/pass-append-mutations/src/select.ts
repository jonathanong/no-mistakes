import sql from "sql-template-strings";
import { read } from "@data-stores/psql";

export async function listTopics(scope: string) {
  const query = sql`SELECT id FROM topics WHERE published = true`;
  query.append(sql` AND tenant_id = ${scope}`);
  query.append(sql` ORDER BY id LIMIT 50`);
  return read(query);
}
