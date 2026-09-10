import sql from "sql-template-strings";
import { write } from "@data-stores/psql";

function buildExpireItemsQuery() {
  const query = sql`WITH stale AS (SELECT id FROM items WHERE expired) `;
  query.append(sql`UPDATE items SET gone = true FROM stale WHERE items.id = stale.id`);
  return query;
}

export async function expireItems() {
  await write(buildExpireItemsQuery());
}
