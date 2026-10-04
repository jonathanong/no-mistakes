import sql from "sql-template-strings";
import { write } from "@example/db";

function buildExpireItemsQuery() {
  const query = sql`WITH logged AS (
    INSERT INTO items (id)
    SELECT input.id
    FROM unnest($1::uuid[]) AS input(id)
    ON CONFLICT (id) DO NOTHING
  ) `;
  query.append(sql`UPDATE items SET gone = true`);
  return query;
}

export async function expireItems() {
  await write(buildExpireItemsQuery());
}
