import sql from "sql-template-strings";
import { write } from "@data-stores/psql";

function buildExpireItemsQuery(now: Date) {
  const query = sql`UPDATE items SET gone = true WHERE expired_at < ${now}`;
  query.append(sql` AND gone = false`);
  return query;
}

export async function expire(now: Date) {
  await write(buildExpireItemsQuery(now));
}
