import sql from "sql-template-strings";
import { write } from "@data-stores/psql";

function buildExpireActivityPubInboxDeliveriesQuery(now: Date) {
  const query = sql`UPDATE activitypub_inbox_deliveries SET expired_at = ${now} WHERE delivered_at IS NULL`;
  query.append(sql` AND created_at < ${now}`);
  return query;
}

export async function expire(now: Date) {
  await write(buildExpireActivityPubInboxDeliveriesQuery(now));
}
