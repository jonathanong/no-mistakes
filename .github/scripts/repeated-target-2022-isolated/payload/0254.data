import sql from "sql-template-strings";
import { write } from "@example/db";

function buildExpireItemsQuery() {
  const query = sql`WITH stale AS (SELECT id FROM items WHERE expired) `;
  query.append(sql`UPDATE items SET gone = true FROM stale WHERE items.id = stale.id`);
  return query;
}

export async function expireItems() {
  await write(buildExpireItemsQuery());
}
